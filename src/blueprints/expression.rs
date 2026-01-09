//! Expression types and AST for parameterized blueprint expressions.
//!
//! This module defines the expression language used in blueprint parameters.
//! Expressions can include literals, parameters, binary/unary operations,
//! conditionals, and function calls.

// HashMap will be used for evaluation context in future implementation
#[allow(unused_imports)]
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
    UnaryOp {
        op: UnaryOp,
        operand: Box<Expr>,
    },
    /// A conditional expression (if condition then true_expr else false_expr)
    Conditional {
        condition: Box<Expr>,
        true_expr: Box<Expr>,
        false_expr: Box<Expr>,
    },
    /// A function call (e.g., min(a, b))
    Function {
        name: String,
        args: Vec<Expr>,
    },
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
