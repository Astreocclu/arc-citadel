//! Expression types and AST for parameterized blueprint expressions.
//!
//! This module defines the expression language used in blueprint parameters.
//! Expressions can include literals, parameters, binary/unary operations,
//! conditionals, and function calls.

use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{alpha1, alphanumeric1, char, multispace0},
    combinator::recognize,
    multi::{fold_many0, many0, separated_list0},
    number::complete::float,
    sequence::{delimited, pair, preceded},
    IResult, Parser,
};
use std::collections::HashMap;

/// Binary operators supported in expressions
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Gt,
    Lt,
    Gte,
    Lte,
    Eq,
    Neq,
    And,
    Or,
}

/// Unary operators
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UnaryOp {
    Neg,
    Not,
}

/// Expression AST node
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    /// A numeric literal (e.g., 42.5)
    Literal(f32),
    /// A parameter reference (e.g., "length")
    Param(String),
    /// A binary operation (e.g., left + right)
    BinOp {
        op: BinOp,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    /// A unary operation (e.g., -x, !condition)
    UnaryOp { op: UnaryOp, operand: Box<Expr> },
    /// A conditional expression (if condition then true_expr else false_expr)
    Conditional {
        condition: Box<Expr>,
        true_expr: Box<Expr>,
        false_expr: Box<Expr>,
    },
    /// A function call (e.g., min(a, b))
    Function { name: String, args: Vec<Expr> },
}

/// Error type for expression evaluation
#[derive(Debug, Clone, PartialEq)]
pub enum EvalError {
    /// Referenced a parameter that doesn't exist in the context
    UnknownParam(String),
    /// Called a function that doesn't exist
    UnknownFunction(String),
    /// Attempted to divide by zero
    DivisionByZero,
    /// Function called with wrong number of arguments
    InvalidArgCount {
        func: String,
        expected: usize,
        got: usize,
    },
}

impl std::fmt::Display for EvalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EvalError::UnknownParam(name) => write!(f, "Unknown parameter: {}", name),
            EvalError::UnknownFunction(name) => write!(f, "Unknown function: {}", name),
            EvalError::DivisionByZero => write!(f, "Division by zero"),
            EvalError::InvalidArgCount {
                func,
                expected,
                got,
            } => {
                write!(
                    f,
                    "Function {} expected {} args, got {}",
                    func, expected, got
                )
            }
        }
    }
}

impl std::error::Error for EvalError {}

/// Error type for expression parsing
#[derive(Debug, Clone, PartialEq)]
pub struct ParseError {
    pub message: String,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Parse error: {}", self.message)
    }
}

impl std::error::Error for ParseError {}

impl ParseError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

// ============================================================================
// Parser Implementation
// ============================================================================

/// Parse an identifier (parameter name or function name)
fn parse_identifier(input: &str) -> IResult<&str, String> {
    recognize(pair(
        alt((alpha1, tag("_"))),
        many0(alt((alphanumeric1, tag("_")))),
    ))
    .map(|s: &str| s.to_string())
    .parse(input)
}

/// Parse a numeric literal (integer or float)
fn parse_literal(input: &str) -> IResult<&str, Expr> {
    float.map(Expr::Literal).parse(input)
}

/// Parse a parameter reference (identifier not followed by '(')
fn parse_param(input: &str) -> IResult<&str, Expr> {
    let (rest, name) = parse_identifier(input)?;
    // Check that this isn't a function call
    let (rest_after_ws, _) = multispace0(rest)?;
    if rest_after_ws.starts_with('(') {
        // This is a function call, not a parameter
        return Err(nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Tag,
        )));
    }
    Ok((rest, Expr::Param(name)))
}

/// Parse a function call: name(arg1, arg2, ...)
fn parse_function_call(input: &str) -> IResult<&str, Expr> {
    let (rest, name) = parse_identifier(input)?;
    let (rest, _) = multispace0(rest)?;
    let (rest, _) = char('(').parse(rest)?;
    let (rest, _) = multispace0(rest)?;
    let (rest, args) =
        separated_list0(delimited(multispace0, char(','), multispace0), parse_expr).parse(rest)?;
    let (rest, _) = multispace0(rest)?;
    let (rest, _) = char(')').parse(rest)?;
    Ok((rest, Expr::Function { name, args }))
}

/// Parse an atom: literal, param, function call, or parenthesized expression
fn parse_atom(input: &str) -> IResult<&str, Expr> {
    let (input, _) = multispace0(input)?;
    alt((
        // Parenthesized expression
        delimited(
            char('('),
            delimited(multispace0, parse_expr, multispace0),
            char(')'),
        ),
        // Function call (must come before param because they both start with identifier)
        parse_function_call,
        // Parameter reference
        parse_param,
        // Numeric literal
        parse_literal,
    ))
    .parse(input)
}

/// Parse unary operators: -, !
fn parse_unary(input: &str) -> IResult<&str, Expr> {
    let (input, _) = multispace0(input)?;
    alt((
        // Unary negation
        preceded(char('-'), parse_unary).map(|expr| Expr::UnaryOp {
            op: UnaryOp::Neg,
            operand: Box::new(expr),
        }),
        // Logical not
        preceded(char('!'), parse_unary).map(|expr| Expr::UnaryOp {
            op: UnaryOp::Not,
            operand: Box::new(expr),
        }),
        // Or just an atom
        parse_atom,
    ))
    .parse(input)
}

/// Parse multiplicative operators: *, /, %
fn parse_factor(input: &str) -> IResult<&str, Expr> {
    let (input, init) = parse_unary(input)?;

    fold_many0(
        pair(
            delimited(
                multispace0,
                alt((char('*'), char('/'), char('%'))),
                multispace0,
            ),
            parse_unary,
        ),
        move || init.clone(),
        |acc, (op_char, val)| {
            let op = match op_char {
                '*' => BinOp::Mul,
                '/' => BinOp::Div,
                '%' => BinOp::Mod,
                _ => unreachable!(),
            };
            Expr::BinOp {
                op,
                left: Box::new(acc),
                right: Box::new(val),
            }
        },
    )
    .parse(input)
}

/// Parse additive operators: +, -
fn parse_term(input: &str) -> IResult<&str, Expr> {
    let (input, init) = parse_factor(input)?;

    fold_many0(
        pair(
            delimited(multispace0, alt((char('+'), char('-'))), multispace0),
            parse_factor,
        ),
        move || init.clone(),
        |acc, (op_char, val)| {
            let op = match op_char {
                '+' => BinOp::Add,
                '-' => BinOp::Sub,
                _ => unreachable!(),
            };
            Expr::BinOp {
                op,
                left: Box::new(acc),
                right: Box::new(val),
            }
        },
    )
    .parse(input)
}

/// Parse comparison operators: >, <, >=, <=, ==, !=
fn parse_comparison(input: &str) -> IResult<&str, Expr> {
    let (input, init) = parse_term(input)?;

    fold_many0(
        pair(
            delimited(
                multispace0,
                alt((
                    tag(">=").map(|_| BinOp::Gte),
                    tag("<=").map(|_| BinOp::Lte),
                    tag("==").map(|_| BinOp::Eq),
                    tag("!=").map(|_| BinOp::Neq),
                    tag(">").map(|_| BinOp::Gt),
                    tag("<").map(|_| BinOp::Lt),
                )),
                multispace0,
            ),
            parse_term,
        ),
        move || init.clone(),
        |acc, (op, val)| Expr::BinOp {
            op,
            left: Box::new(acc),
            right: Box::new(val),
        },
    )
    .parse(input)
}

/// Parse logical AND: &&
fn parse_and(input: &str) -> IResult<&str, Expr> {
    let (input, init) = parse_comparison(input)?;

    fold_many0(
        preceded(
            delimited(multispace0, tag("&&"), multispace0),
            parse_comparison,
        ),
        move || init.clone(),
        |acc, val| Expr::BinOp {
            op: BinOp::And,
            left: Box::new(acc),
            right: Box::new(val),
        },
    )
    .parse(input)
}

/// Parse logical OR: ||
fn parse_or(input: &str) -> IResult<&str, Expr> {
    let (input, init) = parse_and(input)?;

    fold_many0(
        preceded(delimited(multispace0, tag("||"), multispace0), parse_and),
        move || init.clone(),
        |acc, val| Expr::BinOp {
            op: BinOp::Or,
            left: Box::new(acc),
            right: Box::new(val),
        },
    )
    .parse(input)
}

/// Parse conditional: condition ? true_expr : false_expr
/// Also supports: if condition then true_expr else false_expr
fn parse_conditional(input: &str) -> IResult<&str, Expr> {
    let (input, _) = multispace0(input)?;

    // Try "if ... then ... else ..." syntax first
    if let Ok((rest, expr)) = parse_if_then_else(input) {
        return Ok((rest, expr));
    }

    // Otherwise, parse the condition part
    let (input, condition) = parse_or(input)?;

    // Check for ternary operator
    let (input, _) = multispace0(input)?;
    if let Ok((rest, _)) = char::<_, nom::error::Error<&str>>('?').parse(input) {
        let (rest, _) = multispace0(rest)?;
        let (rest, true_expr) = parse_conditional(rest)?;
        let (rest, _) = multispace0(rest)?;
        let (rest, _) = char(':').parse(rest)?;
        let (rest, _) = multispace0(rest)?;
        let (rest, false_expr) = parse_conditional(rest)?;
        Ok((
            rest,
            Expr::Conditional {
                condition: Box::new(condition),
                true_expr: Box::new(true_expr),
                false_expr: Box::new(false_expr),
            },
        ))
    } else {
        Ok((input, condition))
    }
}

/// Parse "if condition then true_expr else false_expr" syntax
fn parse_if_then_else(input: &str) -> IResult<&str, Expr> {
    let (rest, _) = tag("if").parse(input)?;
    let (rest, _) = multispace0(rest)?;

    // We need to parse until we see "then", so we'll parse the condition
    // by repeatedly consuming until we find "then"
    let (rest, condition) = parse_condition_until_then(rest)?;

    let (rest, _) = multispace0(rest)?;
    let (rest, _) = tag("then").parse(rest)?;
    let (rest, _) = multispace0(rest)?;

    let (rest, true_expr) = parse_expr_until_else(rest)?;

    let (rest, _) = multispace0(rest)?;
    let (rest, _) = tag("else").parse(rest)?;
    let (rest, _) = multispace0(rest)?;

    let (rest, false_expr) = parse_conditional(rest)?;

    Ok((
        rest,
        Expr::Conditional {
            condition: Box::new(condition),
            true_expr: Box::new(true_expr),
            false_expr: Box::new(false_expr),
        },
    ))
}

/// Parse the condition part of "if ... then ..." (stops before "then")
fn parse_condition_until_then(input: &str) -> IResult<&str, Expr> {
    // Parse using parse_or but we need to be careful not to consume "then"
    // The trick is that "then" won't be parsed as a valid operator
    parse_or(input)
}

/// Parse the true expression part of "if ... then ... else ..." (stops before "else")
fn parse_expr_until_else(input: &str) -> IResult<&str, Expr> {
    // Similar to above, "else" won't be parsed as a valid expression part
    parse_or(input)
}

/// Entry point for parsing expressions
fn parse_expr(input: &str) -> IResult<&str, Expr> {
    parse_conditional(input)
}

impl Expr {
    /// Parse a string into an expression AST
    pub fn parse(input: &str) -> Result<Expr, ParseError> {
        let trimmed = input.trim();
        match parse_expr(trimmed) {
            Ok(("", expr)) => Ok(expr),
            Ok((remaining, _)) => Err(ParseError::new(format!("Unparsed input: '{}'", remaining))),
            Err(e) => Err(ParseError::new(format!("Parse error: {:?}", e))),
        }
    }

    /// Evaluate the expression with the given parameter values
    pub fn evaluate(&self, params: &HashMap<String, f32>) -> Result<f32, EvalError> {
        match self {
            Expr::Literal(val) => Ok(*val),

            Expr::Param(name) => params
                .get(name)
                .copied()
                .ok_or_else(|| EvalError::UnknownParam(name.clone())),

            Expr::BinOp { op, left, right } => {
                let l = left.evaluate(params)?;
                let r = right.evaluate(params)?;
                Ok(match op {
                    BinOp::Add => l + r,
                    BinOp::Sub => l - r,
                    BinOp::Mul => l * r,
                    BinOp::Div => {
                        if r == 0.0 {
                            return Err(EvalError::DivisionByZero);
                        }
                        l / r
                    }
                    BinOp::Mod => {
                        if r == 0.0 {
                            return Err(EvalError::DivisionByZero);
                        }
                        l % r
                    }
                    BinOp::Gt => {
                        if l > r {
                            1.0
                        } else {
                            0.0
                        }
                    }
                    BinOp::Lt => {
                        if l < r {
                            1.0
                        } else {
                            0.0
                        }
                    }
                    BinOp::Gte => {
                        if l >= r {
                            1.0
                        } else {
                            0.0
                        }
                    }
                    BinOp::Lte => {
                        if l <= r {
                            1.0
                        } else {
                            0.0
                        }
                    }
                    BinOp::Eq => {
                        if (l - r).abs() < f32::EPSILON {
                            1.0
                        } else {
                            0.0
                        }
                    }
                    BinOp::Neq => {
                        if (l - r).abs() >= f32::EPSILON {
                            1.0
                        } else {
                            0.0
                        }
                    }
                    BinOp::And => {
                        if l != 0.0 && r != 0.0 {
                            1.0
                        } else {
                            0.0
                        }
                    }
                    BinOp::Or => {
                        if l != 0.0 || r != 0.0 {
                            1.0
                        } else {
                            0.0
                        }
                    }
                })
            }

            Expr::UnaryOp { op, operand } => {
                let val = operand.evaluate(params)?;
                Ok(match op {
                    UnaryOp::Neg => -val,
                    UnaryOp::Not => {
                        if val == 0.0 {
                            1.0
                        } else {
                            0.0
                        }
                    }
                })
            }

            Expr::Conditional {
                condition,
                true_expr,
                false_expr,
            } => {
                let cond = condition.evaluate(params)?;
                if cond != 0.0 {
                    true_expr.evaluate(params)
                } else {
                    false_expr.evaluate(params)
                }
            }

            Expr::Function { name, args } => {
                let evaluated_args: Result<Vec<f32>, EvalError> =
                    args.iter().map(|a| a.evaluate(params)).collect();
                let evaluated_args = evaluated_args?;

                match name.as_str() {
                    // Single-argument functions
                    "ceil" => {
                        if evaluated_args.len() != 1 {
                            return Err(EvalError::InvalidArgCount {
                                func: name.clone(),
                                expected: 1,
                                got: evaluated_args.len(),
                            });
                        }
                        Ok(evaluated_args[0].ceil())
                    }
                    "floor" => {
                        if evaluated_args.len() != 1 {
                            return Err(EvalError::InvalidArgCount {
                                func: name.clone(),
                                expected: 1,
                                got: evaluated_args.len(),
                            });
                        }
                        Ok(evaluated_args[0].floor())
                    }
                    "sqrt" => {
                        if evaluated_args.len() != 1 {
                            return Err(EvalError::InvalidArgCount {
                                func: name.clone(),
                                expected: 1,
                                got: evaluated_args.len(),
                            });
                        }
                        Ok(evaluated_args[0].sqrt())
                    }
                    "abs" => {
                        if evaluated_args.len() != 1 {
                            return Err(EvalError::InvalidArgCount {
                                func: name.clone(),
                                expected: 1,
                                got: evaluated_args.len(),
                            });
                        }
                        Ok(evaluated_args[0].abs())
                    }
                    // Two-argument functions
                    "min" => {
                        if evaluated_args.len() != 2 {
                            return Err(EvalError::InvalidArgCount {
                                func: name.clone(),
                                expected: 2,
                                got: evaluated_args.len(),
                            });
                        }
                        Ok(evaluated_args[0].min(evaluated_args[1]))
                    }
                    "max" => {
                        if evaluated_args.len() != 2 {
                            return Err(EvalError::InvalidArgCount {
                                func: name.clone(),
                                expected: 2,
                                got: evaluated_args.len(),
                            });
                        }
                        Ok(evaluated_args[0].max(evaluated_args[1]))
                    }
                    _ => Err(EvalError::UnknownFunction(name.clone())),
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_literal_parsing() {
        let result = Expr::parse("42.5");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Expr::Literal(42.5));
    }

    #[test]
    fn test_integer_literal_parsing() {
        let result = Expr::parse("42");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Expr::Literal(42.0));
    }

    #[test]
    fn test_param_parsing() {
        let result = Expr::parse("length");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Expr::Param("length".to_string()));
    }

    #[test]
    fn test_param_with_underscore() {
        let result = Expr::parse("max_height");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Expr::Param("max_height".to_string()));
    }

    #[test]
    fn test_simple_addition() {
        let result = Expr::parse("a + b");
        assert!(result.is_ok());
        match result.unwrap() {
            Expr::BinOp {
                op: BinOp::Add,
                left,
                right,
            } => {
                assert_eq!(*left, Expr::Param("a".to_string()));
                assert_eq!(*right, Expr::Param("b".to_string()));
            }
            other => panic!("Expected BinOp Add, got {:?}", other),
        }
    }

    #[test]
    fn test_simple_subtraction() {
        let result = Expr::parse("a - b");
        assert!(result.is_ok());
        match result.unwrap() {
            Expr::BinOp {
                op: BinOp::Sub,
                left,
                right,
            } => {
                assert_eq!(*left, Expr::Param("a".to_string()));
                assert_eq!(*right, Expr::Param("b".to_string()));
            }
            other => panic!("Expected BinOp Sub, got {:?}", other),
        }
    }

    #[test]
    fn test_simple_multiplication() {
        let result = Expr::parse("length * height");
        assert!(result.is_ok());
        match result.unwrap() {
            Expr::BinOp {
                op: BinOp::Mul,
                left,
                right,
            } => {
                assert_eq!(*left, Expr::Param("length".to_string()));
                assert_eq!(*right, Expr::Param("height".to_string()));
            }
            other => panic!("Expected BinOp Mul, got {:?}", other),
        }
    }

    #[test]
    fn test_simple_division() {
        let result = Expr::parse("a / b");
        assert!(result.is_ok());
        match result.unwrap() {
            Expr::BinOp {
                op: BinOp::Div,
                left,
                right,
            } => {
                assert_eq!(*left, Expr::Param("a".to_string()));
                assert_eq!(*right, Expr::Param("b".to_string()));
            }
            other => panic!("Expected BinOp Div, got {:?}", other),
        }
    }

    #[test]
    fn test_operator_precedence_mul_over_add() {
        // a + b * c should parse as a + (b * c)
        let result = Expr::parse("a + b * c");
        assert!(result.is_ok());
        match result.unwrap() {
            Expr::BinOp {
                op: BinOp::Add,
                left,
                right,
            } => {
                assert_eq!(*left, Expr::Param("a".to_string()));
                match *right {
                    Expr::BinOp {
                        op: BinOp::Mul,
                        left: inner_left,
                        right: inner_right,
                    } => {
                        assert_eq!(*inner_left, Expr::Param("b".to_string()));
                        assert_eq!(*inner_right, Expr::Param("c".to_string()));
                    }
                    other => panic!("Expected inner BinOp Mul, got {:?}", other),
                }
            }
            other => panic!("Expected BinOp Add, got {:?}", other),
        }
    }

    #[test]
    fn test_parentheses_override_precedence() {
        // (a + b) * c should parse as (a + b) * c
        let result = Expr::parse("(a + b) * c");
        assert!(result.is_ok());
        match result.unwrap() {
            Expr::BinOp {
                op: BinOp::Mul,
                left,
                right,
            } => {
                match *left {
                    Expr::BinOp {
                        op: BinOp::Add,
                        left: inner_left,
                        right: inner_right,
                    } => {
                        assert_eq!(*inner_left, Expr::Param("a".to_string()));
                        assert_eq!(*inner_right, Expr::Param("b".to_string()));
                    }
                    other => panic!("Expected inner BinOp Add, got {:?}", other),
                }
                assert_eq!(*right, Expr::Param("c".to_string()));
            }
            other => panic!("Expected BinOp Mul, got {:?}", other),
        }
    }

    #[test]
    fn test_comparison_operators() {
        let result = Expr::parse("a > b");
        assert!(result.is_ok());
        match result.unwrap() {
            Expr::BinOp {
                op: BinOp::Gt,
                left,
                right,
            } => {
                assert_eq!(*left, Expr::Param("a".to_string()));
                assert_eq!(*right, Expr::Param("b".to_string()));
            }
            other => panic!("Expected BinOp Gt, got {:?}", other),
        }
    }

    #[test]
    fn test_equality_operator() {
        let result = Expr::parse("a == b");
        assert!(result.is_ok());
        match result.unwrap() {
            Expr::BinOp {
                op: BinOp::Eq,
                left,
                right,
            } => {
                assert_eq!(*left, Expr::Param("a".to_string()));
                assert_eq!(*right, Expr::Param("b".to_string()));
            }
            other => panic!("Expected BinOp Eq, got {:?}", other),
        }
    }

    #[test]
    fn test_logical_and() {
        let result = Expr::parse("a && b");
        assert!(result.is_ok());
        match result.unwrap() {
            Expr::BinOp {
                op: BinOp::And,
                left,
                right,
            } => {
                assert_eq!(*left, Expr::Param("a".to_string()));
                assert_eq!(*right, Expr::Param("b".to_string()));
            }
            other => panic!("Expected BinOp And, got {:?}", other),
        }
    }

    #[test]
    fn test_logical_or() {
        let result = Expr::parse("a || b");
        assert!(result.is_ok());
        match result.unwrap() {
            Expr::BinOp {
                op: BinOp::Or,
                left,
                right,
            } => {
                assert_eq!(*left, Expr::Param("a".to_string()));
                assert_eq!(*right, Expr::Param("b".to_string()));
            }
            other => panic!("Expected BinOp Or, got {:?}", other),
        }
    }

    #[test]
    fn test_unary_negation() {
        let result = Expr::parse("-x");
        assert!(result.is_ok());
        match result.unwrap() {
            Expr::UnaryOp {
                op: UnaryOp::Neg,
                operand,
            } => {
                assert_eq!(*operand, Expr::Param("x".to_string()));
            }
            other => panic!("Expected UnaryOp Neg, got {:?}", other),
        }
    }

    #[test]
    fn test_unary_not() {
        let result = Expr::parse("!condition");
        assert!(result.is_ok());
        match result.unwrap() {
            Expr::UnaryOp {
                op: UnaryOp::Not,
                operand,
            } => {
                assert_eq!(*operand, Expr::Param("condition".to_string()));
            }
            other => panic!("Expected UnaryOp Not, got {:?}", other),
        }
    }

    #[test]
    fn test_function_call_single_arg() {
        let result = Expr::parse("abs(x)");
        assert!(result.is_ok());
        match result.unwrap() {
            Expr::Function { name, args } => {
                assert_eq!(name, "abs");
                assert_eq!(args.len(), 1);
                assert_eq!(args[0], Expr::Param("x".to_string()));
            }
            other => panic!("Expected Function, got {:?}", other),
        }
    }

    #[test]
    fn test_function_call_multiple_args() {
        let result = Expr::parse("min(a, b)");
        assert!(result.is_ok());
        match result.unwrap() {
            Expr::Function { name, args } => {
                assert_eq!(name, "min");
                assert_eq!(args.len(), 2);
                assert_eq!(args[0], Expr::Param("a".to_string()));
                assert_eq!(args[1], Expr::Param("b".to_string()));
            }
            other => panic!("Expected Function, got {:?}", other),
        }
    }

    #[test]
    fn test_conditional_expression() {
        let result = Expr::parse("if x > 0 then x else -x");
        assert!(result.is_ok());
        match result.unwrap() {
            Expr::Conditional {
                condition,
                true_expr,
                false_expr,
            } => {
                match *condition {
                    Expr::BinOp {
                        op: BinOp::Gt,
                        left,
                        right,
                    } => {
                        assert_eq!(*left, Expr::Param("x".to_string()));
                        assert_eq!(*right, Expr::Literal(0.0));
                    }
                    other => panic!("Expected condition BinOp Gt, got {:?}", other),
                }
                assert_eq!(*true_expr, Expr::Param("x".to_string()));
                match *false_expr {
                    Expr::UnaryOp {
                        op: UnaryOp::Neg,
                        operand,
                    } => {
                        assert_eq!(*operand, Expr::Param("x".to_string()));
                    }
                    other => panic!("Expected false_expr UnaryOp Neg, got {:?}", other),
                }
            }
            other => panic!("Expected Conditional, got {:?}", other),
        }
    }

    #[test]
    fn test_complex_expression() {
        // base_damage * (1 + strength_modifier)
        let result = Expr::parse("base_damage * (1 + strength_modifier)");
        assert!(result.is_ok());
        match result.unwrap() {
            Expr::BinOp {
                op: BinOp::Mul,
                left,
                right,
            } => {
                assert_eq!(*left, Expr::Param("base_damage".to_string()));
                match *right {
                    Expr::BinOp {
                        op: BinOp::Add,
                        left: inner_left,
                        right: inner_right,
                    } => {
                        assert_eq!(*inner_left, Expr::Literal(1.0));
                        assert_eq!(*inner_right, Expr::Param("strength_modifier".to_string()));
                    }
                    other => panic!("Expected inner BinOp Add, got {:?}", other),
                }
            }
            other => panic!("Expected BinOp Mul, got {:?}", other),
        }
    }

    #[test]
    fn test_nested_function_calls() {
        let result = Expr::parse("max(min(a, b), c)");
        assert!(result.is_ok());
        match result.unwrap() {
            Expr::Function { name, args } => {
                assert_eq!(name, "max");
                assert_eq!(args.len(), 2);
                match &args[0] {
                    Expr::Function {
                        name: inner_name,
                        args: inner_args,
                    } => {
                        assert_eq!(inner_name, "min");
                        assert_eq!(inner_args.len(), 2);
                    }
                    other => panic!("Expected inner Function, got {:?}", other),
                }
                assert_eq!(args[1], Expr::Param("c".to_string()));
            }
            other => panic!("Expected Function, got {:?}", other),
        }
    }

    #[test]
    fn test_whitespace_handling() {
        let result = Expr::parse("  a   +   b  ");
        assert!(result.is_ok());
        match result.unwrap() {
            Expr::BinOp {
                op: BinOp::Add,
                left,
                right,
            } => {
                assert_eq!(*left, Expr::Param("a".to_string()));
                assert_eq!(*right, Expr::Param("b".to_string()));
            }
            other => panic!("Expected BinOp Add, got {:?}", other),
        }
    }

    #[test]
    fn test_eval_error_display() {
        let err = EvalError::UnknownParam("foo".to_string());
        assert_eq!(format!("{}", err), "Unknown parameter: foo");

        let err = EvalError::UnknownFunction("bar".to_string());
        assert_eq!(format!("{}", err), "Unknown function: bar");

        let err = EvalError::DivisionByZero;
        assert_eq!(format!("{}", err), "Division by zero");

        let err = EvalError::InvalidArgCount {
            func: "min".to_string(),
            expected: 2,
            got: 1,
        };
        assert_eq!(format!("{}", err), "Function min expected 2 args, got 1");
    }
}
