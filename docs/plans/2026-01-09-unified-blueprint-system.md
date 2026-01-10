# Unified Parameterized Blueprint System Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Implement a unified system where all placeable objects (natural features like trees/rocks and constructed things like walls/buildings) use the same parameterized Blueprint structure with expression-based property formulas.

**Architecture:** Data-driven TOML blueprints loaded via serde, with a nom-based expression parser that compiles formulas at load time and evaluates them at instantiation. BlueprintInstance stores cached evaluated properties. Natural features spawn instantly; constructed features go through ConstructionSite stages.

**Tech Stack:** Rust, nom (expression parsing), serde + toml (data loading), glam (Vec2/geometry)

---

## Prerequisites

**Dependencies to add to Cargo.toml:**
```toml
nom = "8"  # Expression parsing
```

**Existing dependencies we'll use:**
- `serde` with derive (already present)
- `toml = "0.8"` (already present)
- `glam = "0.25"` (already present)
- `uuid` (already present for IDs)
- `ahash` (already present for HashMaps)

---

## Task 1: Add nom Dependency

**Files:**
- Modify: `Cargo.toml`

**Step 1: Add nom to dependencies**

```toml
# Add under [dependencies]
nom = "8"
```

**Step 2: Verify compilation**

Run: `cargo check`
Expected: Compilation succeeds with nom available

**Step 3: Commit**

```bash
git add Cargo.toml
git commit -m "deps: add nom for expression parsing"
```

---

## Task 2: Expression Parser - Core Types

**Files:**
- Create: `src/blueprints/mod.rs`
- Create: `src/blueprints/expression.rs`
- Modify: `src/lib.rs` (add module)

**Step 1: Write failing test for expression parsing**

Create `src/blueprints/expression.rs`:

```rust
use nom::{
    IResult,
    branch::alt,
    bytes::complete::tag,
    character::complete::{alpha1, alphanumeric1, char, multispace0, one_of},
    combinator::{map, opt, recognize, value},
    multi::{fold_many0, many0_count},
    number::complete::float,
    sequence::{delimited, pair, preceded, tuple},
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
    Literal(f32),
    Param(String),
    BinOp {
        op: BinOp,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    UnaryOp {
        op: UnaryOp,
        operand: Box<Expr>,
    },
    Conditional {
        condition: Box<Expr>,
        true_expr: Box<Expr>,
        false_expr: Box<Expr>,
    },
    Function {
        name: String,
        args: Vec<Expr>,
    },
}

/// Error type for expression evaluation
#[derive(Debug, Clone, PartialEq)]
pub enum EvalError {
    UnknownParam(String),
    UnknownFunction(String),
    DivisionByZero,
    InvalidArgCount { func: String, expected: usize, got: usize },
}

impl std::fmt::Display for EvalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EvalError::UnknownParam(name) => write!(f, "Unknown parameter: {}", name),
            EvalError::UnknownFunction(name) => write!(f, "Unknown function: {}", name),
            EvalError::DivisionByZero => write!(f, "Division by zero"),
            EvalError::InvalidArgCount { func, expected, got } => {
                write!(f, "Function {} expected {} args, got {}", func, expected, got)
            }
        }
    }
}

impl std::error::Error for EvalError {}

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
    fn test_param_parsing() {
        let result = Expr::parse("length");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Expr::Param("length".to_string()));
    }

    #[test]
    fn test_simple_multiplication() {
        let result = Expr::parse("length * height");
        assert!(result.is_ok());
        match result.unwrap() {
            Expr::BinOp { op: BinOp::Mul, left, right } => {
                assert_eq!(*left, Expr::Param("length".to_string()));
                assert_eq!(*right, Expr::Param("height".to_string()));
            }
            other => panic!("Expected BinOp Mul, got {:?}", other),
        }
    }
}
```

**Step 2: Create module files**

Create `src/blueprints/mod.rs`:

```rust
pub mod expression;

pub use expression::{Expr, BinOp, UnaryOp, EvalError};
```

**Step 3: Add module to lib.rs**

Add to `src/lib.rs`:

```rust
pub mod blueprints;
```

**Step 4: Run test to verify it fails**

Run: `cargo test blueprints::expression::tests::test_literal_parsing`
Expected: FAIL - `Expr::parse` method not implemented

**Step 5: Commit failing test**

```bash
git add src/blueprints/ src/lib.rs
git commit -m "test: add failing expression parser tests"
```

---

## Task 3: Expression Parser - Implementation

**Files:**
- Modify: `src/blueprints/expression.rs`

**Step 1: Implement nom parsers**

Add to `src/blueprints/expression.rs` after the type definitions:

```rust
impl Expr {
    /// Parse an expression string into an AST
    pub fn parse(input: &str) -> Result<Expr, String> {
        match parse_expr(input.trim()) {
            Ok(("", expr)) => Ok(expr),
            Ok((remaining, _)) => Err(format!("Unparsed input remaining: '{}'", remaining)),
            Err(e) => Err(format!("Parse error: {:?}", e)),
        }
    }

    /// Evaluate expression with parameter values
    pub fn evaluate(&self, params: &HashMap<String, f32>) -> Result<f32, EvalError> {
        match self {
            Expr::Literal(v) => Ok(*v),
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
                    BinOp::Mod => l % r,
                    BinOp::Gt => if l > r { 1.0 } else { 0.0 },
                    BinOp::Lt => if l < r { 1.0 } else { 0.0 },
                    BinOp::Gte => if l >= r { 1.0 } else { 0.0 },
                    BinOp::Lte => if l <= r { 1.0 } else { 0.0 },
                    BinOp::Eq => if (l - r).abs() < f32::EPSILON { 1.0 } else { 0.0 },
                    BinOp::Neq => if (l - r).abs() >= f32::EPSILON { 1.0 } else { 0.0 },
                    BinOp::And => if l != 0.0 && r != 0.0 { 1.0 } else { 0.0 },
                    BinOp::Or => if l != 0.0 || r != 0.0 { 1.0 } else { 0.0 },
                })
            }
            Expr::UnaryOp { op, operand } => {
                let v = operand.evaluate(params)?;
                Ok(match op {
                    UnaryOp::Neg => -v,
                    UnaryOp::Not => if v == 0.0 { 1.0 } else { 0.0 },
                })
            }
            Expr::Conditional { condition, true_expr, false_expr } => {
                let cond = condition.evaluate(params)?;
                if cond != 0.0 {
                    true_expr.evaluate(params)
                } else {
                    false_expr.evaluate(params)
                }
            }
            Expr::Function { name, args } => {
                let evaluated: Result<Vec<f32>, _> = args
                    .iter()
                    .map(|a| a.evaluate(params))
                    .collect();
                let args = evaluated?;
                eval_function(name, &args)
            }
        }
    }
}

fn eval_function(name: &str, args: &[f32]) -> Result<f32, EvalError> {
    match name {
        "ceil" => {
            if args.len() != 1 {
                return Err(EvalError::InvalidArgCount {
                    func: name.to_string(),
                    expected: 1,
                    got: args.len(),
                });
            }
            Ok(args[0].ceil())
        }
        "floor" => {
            if args.len() != 1 {
                return Err(EvalError::InvalidArgCount {
                    func: name.to_string(),
                    expected: 1,
                    got: args.len(),
                });
            }
            Ok(args[0].floor())
        }
        "min" => {
            if args.len() != 2 {
                return Err(EvalError::InvalidArgCount {
                    func: name.to_string(),
                    expected: 2,
                    got: args.len(),
                });
            }
            Ok(args[0].min(args[1]))
        }
        "max" => {
            if args.len() != 2 {
                return Err(EvalError::InvalidArgCount {
                    func: name.to_string(),
                    expected: 2,
                    got: args.len(),
                });
            }
            Ok(args[0].max(args[1]))
        }
        "sqrt" => {
            if args.len() != 1 {
                return Err(EvalError::InvalidArgCount {
                    func: name.to_string(),
                    expected: 1,
                    got: args.len(),
                });
            }
            Ok(args[0].sqrt())
        }
        "abs" => {
            if args.len() != 1 {
                return Err(EvalError::InvalidArgCount {
                    func: name.to_string(),
                    expected: 1,
                    got: args.len(),
                });
            }
            Ok(args[0].abs())
        }
        _ => Err(EvalError::UnknownFunction(name.to_string())),
    }
}

// ============ NOM PARSERS ============

fn ws<'a, F, O>(inner: F) -> impl FnMut(&'a str) -> IResult<&'a str, O>
where
    F: FnMut(&'a str) -> IResult<&'a str, O>,
{
    delimited(multispace0, inner, multispace0)
}

fn parse_identifier(input: &str) -> IResult<&str, String> {
    map(
        recognize(pair(
            alt((alpha1, tag("_"))),
            many0_count(alt((alphanumeric1, tag("_")))),
        )),
        |s: &str| s.to_string(),
    )(input)
}

fn parse_literal(input: &str) -> IResult<&str, Expr> {
    map(float, Expr::Literal)(input)
}

fn parse_param(input: &str) -> IResult<&str, Expr> {
    map(parse_identifier, Expr::Param)(input)
}

fn parse_function(input: &str) -> IResult<&str, Expr> {
    let (input, name) = parse_identifier(input)?;
    let (input, _) = ws(char('('))(input)?;
    let (input, first_arg) = opt(parse_expr)(input)?;
    let (input, rest_args) = many0_count(preceded(ws(char(',')), parse_expr))(input);

    // Re-parse to get actual args (simpler approach)
    let (input2, _) = parse_identifier(input)?;
    // Actually, let's do this properly with a fold

    // Restart with simpler approach
    Err(nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::Tag)))
}

fn parse_atom(input: &str) -> IResult<&str, Expr> {
    alt((
        parse_literal,
        parse_function_call,
        parse_param,
        delimited(ws(char('(')), parse_expr, ws(char(')'))),
    ))(input)
}

fn parse_function_call(input: &str) -> IResult<&str, Expr> {
    let (input, name) = parse_identifier(input)?;
    let (input, _) = char('(')(input)?;
    let (input, _) = multispace0(input)?;

    // Parse first arg if exists
    let (input, args) = parse_function_args(input)?;

    let (input, _) = multispace0(input)?;
    let (input, _) = char(')')(input)?;

    Ok((input, Expr::Function { name, args }))
}

fn parse_function_args(input: &str) -> IResult<&str, Vec<Expr>> {
    let (input, first) = opt(parse_expr)(input)?;
    match first {
        None => Ok((input, vec![])),
        Some(first_expr) => {
            let (input, mut rest) = many0(preceded(
                tuple((multispace0, char(','), multispace0)),
                parse_expr,
            ))(input)?;
            let mut args = vec![first_expr];
            args.append(&mut rest);
            Ok((input, args))
        }
    }
}

fn parse_unary(input: &str) -> IResult<&str, Expr> {
    alt((
        map(
            preceded(ws(char('-')), parse_unary),
            |e| Expr::UnaryOp { op: UnaryOp::Neg, operand: Box::new(e) },
        ),
        map(
            preceded(ws(char('!')), parse_unary),
            |e| Expr::UnaryOp { op: UnaryOp::Not, operand: Box::new(e) },
        ),
        parse_atom,
    ))(input)
}

fn parse_factor(input: &str) -> IResult<&str, Expr> {
    let (input, init) = parse_unary(input)?;
    fold_many0(
        pair(
            ws(alt((
                value(BinOp::Mul, char('*')),
                value(BinOp::Div, char('/')),
                value(BinOp::Mod, char('%')),
            ))),
            parse_unary,
        ),
        move || init.clone(),
        |acc, (op, val)| Expr::BinOp {
            op,
            left: Box::new(acc),
            right: Box::new(val),
        },
    )(input)
}

fn parse_term(input: &str) -> IResult<&str, Expr> {
    let (input, init) = parse_factor(input)?;
    fold_many0(
        pair(
            ws(alt((
                value(BinOp::Add, char('+')),
                value(BinOp::Sub, char('-')),
            ))),
            parse_factor,
        ),
        move || init.clone(),
        |acc, (op, val)| Expr::BinOp {
            op,
            left: Box::new(acc),
            right: Box::new(val),
        },
    )(input)
}

fn parse_comparison(input: &str) -> IResult<&str, Expr> {
    let (input, init) = parse_term(input)?;
    fold_many0(
        pair(
            ws(alt((
                value(BinOp::Gte, tag(">=")),
                value(BinOp::Lte, tag("<=")),
                value(BinOp::Gt, char('>')),
                value(BinOp::Lt, char('<')),
                value(BinOp::Eq, tag("==")),
                value(BinOp::Neq, tag("!=")),
            ))),
            parse_term,
        ),
        move || init.clone(),
        |acc, (op, val)| Expr::BinOp {
            op,
            left: Box::new(acc),
            right: Box::new(val),
        },
    )(input)
}

fn parse_and(input: &str) -> IResult<&str, Expr> {
    let (input, init) = parse_comparison(input)?;
    fold_many0(
        preceded(ws(tag("&&")), parse_comparison),
        move || init.clone(),
        |acc, val| Expr::BinOp {
            op: BinOp::And,
            left: Box::new(acc),
            right: Box::new(val),
        },
    )(input)
}

fn parse_or(input: &str) -> IResult<&str, Expr> {
    let (input, init) = parse_and(input)?;
    fold_many0(
        preceded(ws(tag("||")), parse_and),
        move || init.clone(),
        |acc, val| Expr::BinOp {
            op: BinOp::Or,
            left: Box::new(acc),
            right: Box::new(val),
        },
    )(input)
}

fn parse_conditional(input: &str) -> IResult<&str, Expr> {
    let (input, condition) = parse_or(input)?;
    let (input, ternary) = opt(tuple((
        ws(char('?')),
        parse_expr,
        ws(char(':')),
        parse_expr,
    )))(input)?;

    match ternary {
        Some((_, true_expr, _, false_expr)) => Ok((
            input,
            Expr::Conditional {
                condition: Box::new(condition),
                true_expr: Box::new(true_expr),
                false_expr: Box::new(false_expr),
            },
        )),
        None => Ok((input, condition)),
    }
}

fn parse_expr(input: &str) -> IResult<&str, Expr> {
    parse_conditional(input)
}
```

**Step 2: Run tests to verify they pass**

Run: `cargo test blueprints::expression::tests`
Expected: All 3 tests pass

**Step 3: Add more comprehensive tests**

Add to the tests module:

```rust
    #[test]
    fn test_complex_expression() {
        let result = Expr::parse("length * height * 80");
        assert!(result.is_ok());
    }

    #[test]
    fn test_function_call() {
        let result = Expr::parse("ceil(length / 2)");
        assert!(result.is_ok());
    }

    #[test]
    fn test_conditional() {
        let result = Expr::parse("height >= 1.5 ? 1.0 : 0.5");
        assert!(result.is_ok());
    }

    #[test]
    fn test_evaluation() {
        let expr = Expr::parse("length * height * 80").unwrap();
        let mut params = HashMap::new();
        params.insert("length".to_string(), 5.0);
        params.insert("height".to_string(), 3.0);

        let result = expr.evaluate(&params).unwrap();
        assert_eq!(result, 1200.0);
    }

    #[test]
    fn test_conditional_evaluation() {
        let expr = Expr::parse("height >= 1.5 ? 1.0 : 0.5").unwrap();

        let mut params = HashMap::new();
        params.insert("height".to_string(), 2.0);
        assert_eq!(expr.evaluate(&params).unwrap(), 1.0);

        params.insert("height".to_string(), 1.0);
        assert_eq!(expr.evaluate(&params).unwrap(), 0.5);
    }

    #[test]
    fn test_function_evaluation() {
        let expr = Expr::parse("ceil(length / 2)").unwrap();
        let mut params = HashMap::new();
        params.insert("length".to_string(), 5.0);

        assert_eq!(expr.evaluate(&params).unwrap(), 3.0);
    }
```

**Step 4: Run all expression tests**

Run: `cargo test blueprints::expression`
Expected: All tests pass

**Step 5: Commit**

```bash
git add src/blueprints/expression.rs
git commit -m "feat: implement expression parser with nom"
```

---

## Task 4: Blueprint Schema Types

**Files:**
- Create: `src/blueprints/schema.rs`
- Modify: `src/blueprints/mod.rs`

**Step 1: Write failing test for blueprint deserialization**

Create `src/blueprints/schema.rs`:

```rust
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use super::expression::Expr;

/// ID for a blueprint template
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BlueprintId(pub u32);

/// Whether this is a natural feature or constructed
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OriginType {
    Natural,
    Constructed,
}

/// Category of the blueprint
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BlueprintCategory {
    // Constructed
    Wall,
    Tower,
    Gate,
    Trench,
    Street,
    Building,
    Furniture,
    // Natural
    Tree,
    Rock,
    Water,
    Vegetation,
    Terrain,
}

/// Parameter type definition
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum ParameterType {
    Float { min: f32, max: f32, default: f32 },
    Int { min: i32, max: i32, default: i32 },
    Bool { default: bool },
}

/// A parameterized blueprint for any placeable thing
#[derive(Debug, Clone, Deserialize)]
pub struct Blueprint {
    pub meta: BlueprintMeta,
    pub parameters: HashMap<String, ParameterType>,
    pub geometry: GeometryFormula,
    #[serde(default)]
    pub stats: Stats,
    #[serde(default)]
    pub construction: Option<ConstructionDef>,
    #[serde(default)]
    pub anchors: Vec<AnchorDef>,
    #[serde(default)]
    pub damage_states: Vec<DamageStateDef>,
    #[serde(default)]
    pub constraints: Vec<ConstraintDef>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BlueprintMeta {
    pub id: String,
    pub name: String,
    pub category: BlueprintCategory,
    #[serde(default)]
    pub origin: OriginType,
    #[serde(default)]
    pub description: String,
}

impl Default for OriginType {
    fn default() -> Self {
        OriginType::Constructed
    }
}

/// Geometry definition using expression strings
#[derive(Debug, Clone, Deserialize)]
pub struct GeometryFormula {
    pub width: String,
    pub depth: String,
    pub height: String,
    #[serde(default = "default_shape")]
    pub shape: String,
}

fn default_shape() -> String {
    "rectangle".to_string()
}

/// Stats containing military and civilian property formulas
#[derive(Debug, Clone, Default, Deserialize)]
pub struct Stats {
    #[serde(default)]
    pub military: MilitaryStats,
    #[serde(default)]
    pub civilian: CivilianStats,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct MilitaryStats {
    #[serde(default = "default_hp")]
    pub max_hp: String,
    #[serde(default)]
    pub hardness: String,
    #[serde(default)]
    pub cover_value: String,
    #[serde(default)]
    pub blocks_movement: String,
    #[serde(default)]
    pub blocks_los: String,
    #[serde(default)]
    pub movement_cost: String,
    #[serde(default)]
    pub flammable: String,
    #[serde(default)]
    pub elevation: String,
}

fn default_hp() -> String {
    "100".to_string()
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct CivilianStats {
    #[serde(default)]
    pub pedestrian_capacity: String,
    #[serde(default)]
    pub cart_accessible: String,
    #[serde(default)]
    pub worker_capacity: String,
    #[serde(default)]
    pub storage_capacity: String,
    #[serde(default)]
    pub prestige_modifier: String,
}

/// Construction definition for constructed origin types
#[derive(Debug, Clone, Deserialize)]
pub struct ConstructionDef {
    pub base_time: String,
    #[serde(default)]
    pub labor_cap: String,
    pub cost: HashMap<String, String>,
    #[serde(default)]
    pub stages: Vec<ConstructionStageDef>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ConstructionStageDef {
    pub id: String,
    pub progress_threshold: f32,
    #[serde(default = "default_height_mult")]
    pub height_multiplier: f32,
    pub visual_state: String,
    #[serde(default)]
    pub overrides: PropertyOverrides,
}

fn default_height_mult() -> f32 {
    1.0
}

/// Anchor point for connecting to other blueprints
#[derive(Debug, Clone, Deserialize)]
pub struct AnchorDef {
    pub name: String,
    pub position: [String; 3],
    pub direction: String,
    #[serde(default)]
    pub tags: Vec<String>,
}

/// Damage state definition
#[derive(Debug, Clone, Deserialize)]
pub struct DamageStateDef {
    pub name: String,
    pub threshold: f32,
    #[serde(default)]
    pub visual_overlay: String,
    #[serde(default)]
    pub overrides: PropertyOverrides,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub creates_breach: bool,
    #[serde(default)]
    pub produces_rubble: bool,
}

/// Property overrides for construction stages and damage states
#[derive(Debug, Clone, Default, Deserialize)]
pub struct PropertyOverrides {
    pub cover_value: Option<f32>,
    pub blocks_movement: Option<bool>,
    pub blocks_los: Option<bool>,
    pub hardness: Option<f32>,
}

/// Constraint that must be satisfied for valid parameters
#[derive(Debug, Clone, Deserialize)]
pub struct ConstraintDef {
    pub description: String,
    pub expression: String,
    pub error_message: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_stone_wall() {
        let toml_str = r#"
[meta]
id = "stone_wall"
name = "Stone Wall"
category = "wall"
origin = "constructed"
description = "A defensive stone wall"

[parameters]
length = { type = "float", min = 2.0, max = 20.0, default = 5.0 }
height = { type = "float", min = 1.0, max = 5.0, default = 2.5 }
thickness = { type = "float", min = 0.4, max = 1.5, default = 0.6 }

[geometry]
width = "length"
depth = "thickness"
height = "height"
shape = "rectangle"

[stats.military]
max_hp = "length * height * 80"
cover_value = "height >= 1.5 ? 1.0 : 0.5"
blocks_movement = "1"
blocks_los = "height >= 1.8 ? 1 : 0"
flammable = "0"

[construction]
base_time = "length * height * 20"
labor_cap = "ceil(length / 2)"

[construction.cost]
stone = "ceil(length * height * 5)"
mortar = "ceil(length * height * 1.5)"

[[construction.stages]]
id = "foundation"
progress_threshold = 0.0
height_multiplier = 0.1
visual_state = "foundation"
overrides = { blocks_movement = false, blocks_los = false }

[[construction.stages]]
id = "complete"
progress_threshold = 1.0
height_multiplier = 1.0
visual_state = "complete"

[[anchors]]
name = "west"
position = ["0", "0", "thickness / 2"]
direction = "west"
tags = ["wall_joint"]

[[anchors]]
name = "east"
position = ["length", "0", "thickness / 2"]
direction = "east"
tags = ["wall_joint"]

[[damage_states]]
name = "intact"
threshold = 0.75

[[damage_states]]
name = "breached"
threshold = 0.25
visual_overlay = "rubble"
creates_breach = true
produces_rubble = true
overrides = { cover_value = 0.5, blocks_movement = false }

[[constraints]]
description = "Minimum length"
expression = "length >= 2"
error_message = "Wall must be at least 2m long"
"#;

        let blueprint: Blueprint = toml::from_str(toml_str).unwrap();

        assert_eq!(blueprint.meta.id, "stone_wall");
        assert_eq!(blueprint.meta.category, BlueprintCategory::Wall);
        assert_eq!(blueprint.meta.origin, OriginType::Constructed);
        assert!(blueprint.construction.is_some());
        assert_eq!(blueprint.anchors.len(), 2);
        assert_eq!(blueprint.damage_states.len(), 2);
    }

    #[test]
    fn test_deserialize_natural() {
        let toml_str = r#"
[meta]
id = "oak_tree"
name = "Oak Tree"
category = "tree"
origin = "natural"

[parameters]
height = { type = "float", min = 8.0, max = 25.0, default = 15.0 }
canopy_radius = { type = "float", min = 3.0, max = 10.0, default = 5.0 }
trunk_radius = { type = "float", min = 0.3, max = 1.5, default = 0.5 }

[geometry]
width = "trunk_radius * 2"
depth = "trunk_radius * 2"
height = "height"
shape = "circle"

[stats.military]
max_hp = "height * 20"
cover_value = "0.5"
blocks_movement = "1"
blocks_los = "1"
flammable = "1"
"#;

        let blueprint: Blueprint = toml::from_str(toml_str).unwrap();

        assert_eq!(blueprint.meta.id, "oak_tree");
        assert_eq!(blueprint.meta.origin, OriginType::Natural);
        assert!(blueprint.construction.is_none());
    }
}
```

**Step 2: Update mod.rs**

Add to `src/blueprints/mod.rs`:

```rust
pub mod expression;
pub mod schema;

pub use expression::{Expr, BinOp, UnaryOp, EvalError};
pub use schema::*;
```

**Step 3: Run tests**

Run: `cargo test blueprints::schema::tests`
Expected: All tests pass

**Step 4: Commit**

```bash
git add src/blueprints/schema.rs src/blueprints/mod.rs
git commit -m "feat: add blueprint TOML schema types with serde"
```

---

## Task 5: Blueprint Instance

**Files:**
- Create: `src/blueprints/instance.rs`
- Modify: `src/blueprints/mod.rs`

**Step 1: Define instance types**

Create `src/blueprints/instance.rs`:

```rust
use glam::Vec2;
use std::collections::HashMap;
use super::schema::{BlueprintId, PropertyOverrides};

/// Unique identifier for an instance
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct InstanceId(pub u64);

/// Tracking who placed this instance and when
#[derive(Debug, Clone)]
pub enum PlacedBy {
    /// Terrain generation during worldgen
    TerrainGen,
    /// NPC polity during history simulation
    HistorySim { polity_id: u32, year: i32 },
    /// Player during gameplay
    Gameplay { tick: u64 },
}

/// Evaluated military properties (cached at instantiation)
#[derive(Debug, Clone, Default)]
pub struct MilitaryProperties {
    pub max_hp: f32,
    pub hardness: f32,
    pub cover_value: f32,
    pub blocks_movement: bool,
    pub blocks_los: bool,
    pub movement_cost: f32,
    pub flammable: bool,
    pub elevation: f32,
}

/// Evaluated civilian properties (cached at instantiation)
#[derive(Debug, Clone, Default)]
pub struct CivilianProperties {
    pub pedestrian_capacity: u32,
    pub cart_accessible: bool,
    pub worker_capacity: u32,
    pub storage_capacity: u32,
    pub prestige_modifier: f32,
}

/// Evaluated geometry (cached at instantiation)
#[derive(Debug, Clone)]
pub struct EvaluatedGeometry {
    pub width: f32,
    pub depth: f32,
    pub height: f32,
    pub footprint: Vec<Vec2>,  // Polygon vertices in local space
}

/// A resolved anchor point
#[derive(Debug, Clone)]
pub struct ResolvedAnchor {
    pub name: String,
    pub position: glam::Vec3,
    pub direction: glam::Vec3,
    pub tags: Vec<String>,
}

/// Breach in a damaged structure
#[derive(Debug, Clone)]
pub struct Breach {
    pub position: Vec2,
    pub width: f32,
}

/// A spawned instance of a blueprint
#[derive(Debug, Clone)]
pub struct BlueprintInstance {
    pub id: InstanceId,
    pub blueprint_id: BlueprintId,
    pub blueprint_name: String,

    /// The parameter values used to instantiate
    pub parameters: HashMap<String, f32>,

    /// World position and rotation
    pub position: Vec2,
    pub rotation: f32,

    /// Cached evaluated geometry
    pub geometry: EvaluatedGeometry,

    /// Current HP (mutable during gameplay)
    pub current_hp: f32,
    /// Cached max HP from evaluation
    pub max_hp: f32,
    /// Current damage state name
    pub damage_state: String,
    /// Active breaches
    pub breaches: Vec<Breach>,

    /// Cached military properties
    pub military: MilitaryProperties,
    /// Cached civilian properties
    pub civilian: CivilianProperties,

    /// Resolved anchor points
    pub anchors: Vec<ResolvedAnchor>,

    /// Construction progress (0.0 to 1.0, 1.0 = complete)
    pub construction_progress: f32,
    /// Current construction stage id
    pub construction_stage: Option<String>,

    /// Who placed this and when
    pub placed_by: PlacedBy,
    /// Owner faction (None for natural features)
    pub owner: Option<u32>,
}

impl BlueprintInstance {
    /// Get current HP as a ratio of max HP
    pub fn hp_ratio(&self) -> f32 {
        if self.max_hp == 0.0 {
            1.0
        } else {
            self.current_hp / self.max_hp
        }
    }

    /// Check if this instance is fully constructed
    pub fn is_complete(&self) -> bool {
        self.construction_progress >= 1.0
    }

    /// Apply damage, returns true if destroyed
    pub fn apply_damage(&mut self, amount: f32) -> bool {
        self.current_hp = (self.current_hp - amount).max(0.0);
        self.current_hp <= 0.0
    }

    /// Apply property overrides from damage/construction state
    pub fn apply_overrides(&mut self, overrides: &PropertyOverrides) {
        if let Some(cover) = overrides.cover_value {
            self.military.cover_value = cover;
        }
        if let Some(blocks) = overrides.blocks_movement {
            self.military.blocks_movement = blocks;
        }
        if let Some(los) = overrides.blocks_los {
            self.military.blocks_los = los;
        }
        if let Some(hard) = overrides.hardness {
            self.military.hardness = hard;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hp_ratio() {
        let instance = BlueprintInstance {
            id: InstanceId(1),
            blueprint_id: BlueprintId(1),
            blueprint_name: "test".to_string(),
            parameters: HashMap::new(),
            position: Vec2::ZERO,
            rotation: 0.0,
            geometry: EvaluatedGeometry {
                width: 5.0,
                depth: 1.0,
                height: 3.0,
                footprint: vec![],
            },
            current_hp: 50.0,
            max_hp: 100.0,
            damage_state: "intact".to_string(),
            breaches: vec![],
            military: MilitaryProperties::default(),
            civilian: CivilianProperties::default(),
            anchors: vec![],
            construction_progress: 1.0,
            construction_stage: None,
            placed_by: PlacedBy::TerrainGen,
            owner: None,
        };

        assert_eq!(instance.hp_ratio(), 0.5);
        assert!(instance.is_complete());
    }

    #[test]
    fn test_apply_damage() {
        let mut instance = BlueprintInstance {
            id: InstanceId(1),
            blueprint_id: BlueprintId(1),
            blueprint_name: "test".to_string(),
            parameters: HashMap::new(),
            position: Vec2::ZERO,
            rotation: 0.0,
            geometry: EvaluatedGeometry {
                width: 5.0,
                depth: 1.0,
                height: 3.0,
                footprint: vec![],
            },
            current_hp: 100.0,
            max_hp: 100.0,
            damage_state: "intact".to_string(),
            breaches: vec![],
            military: MilitaryProperties::default(),
            civilian: CivilianProperties::default(),
            anchors: vec![],
            construction_progress: 1.0,
            construction_stage: None,
            placed_by: PlacedBy::Gameplay { tick: 0 },
            owner: Some(1),
        };

        assert!(!instance.apply_damage(30.0));
        assert_eq!(instance.current_hp, 70.0);

        assert!(instance.apply_damage(100.0));
        assert_eq!(instance.current_hp, 0.0);
    }
}
```

**Step 2: Update mod.rs**

Update `src/blueprints/mod.rs`:

```rust
pub mod expression;
pub mod schema;
pub mod instance;

pub use expression::{Expr, BinOp, UnaryOp, EvalError};
pub use schema::*;
pub use instance::*;
```

**Step 3: Run tests**

Run: `cargo test blueprints::instance::tests`
Expected: All tests pass

**Step 4: Commit**

```bash
git add src/blueprints/instance.rs src/blueprints/mod.rs
git commit -m "feat: add BlueprintInstance with cached properties"
```

---

## Task 6: Blueprint Registry

**Files:**
- Create: `src/blueprints/registry.rs`
- Modify: `src/blueprints/mod.rs`

**Step 1: Create registry with loading and instantiation**

Create `src/blueprints/registry.rs`:

```rust
use std::collections::HashMap;
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use glam::Vec2;

use super::expression::Expr;
use super::schema::*;
use super::instance::*;

/// Error type for blueprint operations
#[derive(Debug)]
pub enum BlueprintError {
    IoError(std::io::Error),
    ParseError(String),
    NotFound(String),
    ValidationError(Vec<String>),
    ExpressionError(super::EvalError),
}

impl std::fmt::Display for BlueprintError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BlueprintError::IoError(e) => write!(f, "IO error: {}", e),
            BlueprintError::ParseError(e) => write!(f, "Parse error: {}", e),
            BlueprintError::NotFound(name) => write!(f, "Blueprint not found: {}", name),
            BlueprintError::ValidationError(errs) => {
                write!(f, "Validation errors: {}", errs.join(", "))
            }
            BlueprintError::ExpressionError(e) => write!(f, "Expression error: {}", e),
        }
    }
}

impl std::error::Error for BlueprintError {}

impl From<std::io::Error> for BlueprintError {
    fn from(e: std::io::Error) -> Self {
        BlueprintError::IoError(e)
    }
}

impl From<super::EvalError> for BlueprintError {
    fn from(e: super::EvalError) -> Self {
        BlueprintError::ExpressionError(e)
    }
}

/// Registry that holds all loaded blueprints
pub struct BlueprintRegistry {
    blueprints: HashMap<BlueprintId, Blueprint>,
    by_name: HashMap<String, BlueprintId>,
    by_category: HashMap<BlueprintCategory, Vec<BlueprintId>>,
    next_id: u32,
    next_instance_id: AtomicU64,
}

impl BlueprintRegistry {
    pub fn new() -> Self {
        Self {
            blueprints: HashMap::new(),
            by_name: HashMap::new(),
            by_category: HashMap::new(),
            next_id: 0,
            next_instance_id: AtomicU64::new(1),
        }
    }

    /// Register a blueprint and return its ID
    pub fn register(&mut self, mut blueprint: Blueprint) -> BlueprintId {
        let id = BlueprintId(self.next_id);
        self.next_id += 1;

        let name = blueprint.meta.id.clone();
        let category = blueprint.meta.category;

        self.by_name.insert(name, id);
        self.by_category
            .entry(category)
            .or_insert_with(Vec::new)
            .push(id);
        self.blueprints.insert(id, blueprint);

        id
    }

    /// Load a blueprint from a TOML file
    pub fn load_file(&mut self, path: &Path) -> Result<BlueprintId, BlueprintError> {
        let content = std::fs::read_to_string(path)?;
        let blueprint: Blueprint = toml::from_str(&content)
            .map_err(|e| BlueprintError::ParseError(e.to_string()))?;
        Ok(self.register(blueprint))
    }

    /// Load all blueprints from a directory (recursively)
    pub fn load_directory(&mut self, path: &Path) -> Result<Vec<BlueprintId>, BlueprintError> {
        let mut loaded = Vec::new();

        if path.is_dir() {
            for entry in std::fs::read_dir(path)? {
                let entry = entry?;
                let entry_path = entry.path();

                if entry_path.is_dir() {
                    loaded.extend(self.load_directory(&entry_path)?);
                } else if entry_path.extension().map_or(false, |ext| ext == "toml") {
                    loaded.push(self.load_file(&entry_path)?);
                }
            }
        }

        Ok(loaded)
    }

    /// Get a blueprint by ID
    pub fn get(&self, id: BlueprintId) -> Option<&Blueprint> {
        self.blueprints.get(&id)
    }

    /// Get a blueprint by name
    pub fn get_by_name(&self, name: &str) -> Option<&Blueprint> {
        self.by_name.get(name).and_then(|id| self.blueprints.get(id))
    }

    /// Get blueprint ID by name
    pub fn id_by_name(&self, name: &str) -> Option<BlueprintId> {
        self.by_name.get(name).copied()
    }

    /// Get all blueprints in a category
    pub fn get_by_category(&self, category: BlueprintCategory) -> Vec<&Blueprint> {
        self.by_category
            .get(&category)
            .map(|ids| ids.iter().filter_map(|id| self.blueprints.get(id)).collect())
            .unwrap_or_default()
    }

    /// Validate parameters against blueprint constraints
    pub fn validate_params(
        &self,
        blueprint_id: BlueprintId,
        params: &HashMap<String, f32>,
    ) -> Result<(), BlueprintError> {
        let blueprint = self
            .get(blueprint_id)
            .ok_or_else(|| BlueprintError::NotFound(format!("ID {:?}", blueprint_id)))?;

        let mut errors = Vec::new();

        // Check parameter ranges
        for (name, param_type) in &blueprint.parameters {
            if let Some(&value) = params.get(name) {
                match param_type {
                    ParameterType::Float { min, max, .. } => {
                        if value < *min || value > *max {
                            errors.push(format!(
                                "{} = {} out of range [{}, {}]",
                                name, value, min, max
                            ));
                        }
                    }
                    ParameterType::Int { min, max, .. } => {
                        let int_val = value as i32;
                        if int_val < *min || int_val > *max {
                            errors.push(format!(
                                "{} = {} out of range [{}, {}]",
                                name, int_val, min, max
                            ));
                        }
                    }
                    ParameterType::Bool { .. } => {}
                }
            }
        }

        // Check constraints
        for constraint in &blueprint.constraints {
            match Expr::parse(&constraint.expression) {
                Ok(expr) => match expr.evaluate(params) {
                    Ok(result) if result == 0.0 => {
                        errors.push(constraint.error_message.clone());
                    }
                    Err(e) => {
                        errors.push(format!("Constraint eval error: {}", e));
                    }
                    _ => {}
                },
                Err(e) => {
                    errors.push(format!("Constraint parse error: {}", e));
                }
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(BlueprintError::ValidationError(errors))
        }
    }

    /// Create an instance from a blueprint
    pub fn instantiate(
        &self,
        blueprint_id: BlueprintId,
        params: HashMap<String, f32>,
        position: Vec2,
        rotation: f32,
        placed_by: PlacedBy,
        owner: Option<u32>,
    ) -> Result<BlueprintInstance, BlueprintError> {
        let blueprint = self
            .get(blueprint_id)
            .ok_or_else(|| BlueprintError::NotFound(format!("ID {:?}", blueprint_id)))?;

        // Fill in defaults for missing params
        let mut full_params = params.clone();
        for (name, param_type) in &blueprint.parameters {
            if !full_params.contains_key(name) {
                let default = match param_type {
                    ParameterType::Float { default, .. } => *default,
                    ParameterType::Int { default, .. } => *default as f32,
                    ParameterType::Bool { default } => if *default { 1.0 } else { 0.0 },
                };
                full_params.insert(name.clone(), default);
            }
        }

        // Validate
        self.validate_params(blueprint_id, &full_params)?;

        // Evaluate geometry
        let width = eval_expr_str(&blueprint.geometry.width, &full_params)?;
        let depth = eval_expr_str(&blueprint.geometry.depth, &full_params)?;
        let height = eval_expr_str(&blueprint.geometry.height, &full_params)?;

        let geometry = EvaluatedGeometry {
            width,
            depth,
            height,
            footprint: generate_footprint(&blueprint.geometry.shape, width, depth),
        };

        // Evaluate military properties
        let max_hp = eval_expr_str(&blueprint.stats.military.max_hp, &full_params)?;
        let military = MilitaryProperties {
            max_hp,
            hardness: eval_expr_str_or(&blueprint.stats.military.hardness, &full_params, 0.0)?,
            cover_value: eval_expr_str_or(&blueprint.stats.military.cover_value, &full_params, 0.0)?,
            blocks_movement: eval_expr_str_or(&blueprint.stats.military.blocks_movement, &full_params, 0.0)? != 0.0,
            blocks_los: eval_expr_str_or(&blueprint.stats.military.blocks_los, &full_params, 0.0)? != 0.0,
            movement_cost: eval_expr_str_or(&blueprint.stats.military.movement_cost, &full_params, 1.0)?,
            flammable: eval_expr_str_or(&blueprint.stats.military.flammable, &full_params, 0.0)? != 0.0,
            elevation: eval_expr_str_or(&blueprint.stats.military.elevation, &full_params, 0.0)?,
        };

        // Evaluate civilian properties
        let civilian = CivilianProperties {
            pedestrian_capacity: eval_expr_str_or(&blueprint.stats.civilian.pedestrian_capacity, &full_params, 0.0)? as u32,
            cart_accessible: eval_expr_str_or(&blueprint.stats.civilian.cart_accessible, &full_params, 0.0)? != 0.0,
            worker_capacity: eval_expr_str_or(&blueprint.stats.civilian.worker_capacity, &full_params, 0.0)? as u32,
            storage_capacity: eval_expr_str_or(&blueprint.stats.civilian.storage_capacity, &full_params, 0.0)? as u32,
            prestige_modifier: eval_expr_str_or(&blueprint.stats.civilian.prestige_modifier, &full_params, 1.0)?,
        };

        // Evaluate anchors
        let anchors = blueprint
            .anchors
            .iter()
            .map(|a| resolve_anchor(a, &full_params))
            .collect::<Result<Vec<_>, _>>()?;

        // Determine initial construction state
        let (construction_progress, construction_stage) = match blueprint.meta.origin {
            OriginType::Natural => (1.0, None),
            OriginType::Constructed => {
                if blueprint.construction.is_some() {
                    (0.0, Some("foundation".to_string()))
                } else {
                    (1.0, None)
                }
            }
        };

        // Determine initial damage state
        let damage_state = blueprint
            .damage_states
            .first()
            .map(|s| s.name.clone())
            .unwrap_or_else(|| "intact".to_string());

        let instance_id = InstanceId(self.next_instance_id.fetch_add(1, Ordering::SeqCst));

        Ok(BlueprintInstance {
            id: instance_id,
            blueprint_id,
            blueprint_name: blueprint.meta.id.clone(),
            parameters: full_params,
            position,
            rotation,
            geometry,
            current_hp: max_hp,
            max_hp,
            damage_state,
            breaches: vec![],
            military,
            civilian,
            anchors,
            construction_progress,
            construction_stage,
            placed_by,
            owner,
        })
    }
}

impl Default for BlueprintRegistry {
    fn default() -> Self {
        Self::new()
    }
}

fn eval_expr_str(expr_str: &str, params: &HashMap<String, f32>) -> Result<f32, BlueprintError> {
    if expr_str.is_empty() {
        return Ok(0.0);
    }
    let expr = Expr::parse(expr_str).map_err(|e| BlueprintError::ParseError(e))?;
    Ok(expr.evaluate(params)?)
}

fn eval_expr_str_or(
    expr_str: &str,
    params: &HashMap<String, f32>,
    default: f32,
) -> Result<f32, BlueprintError> {
    if expr_str.is_empty() {
        return Ok(default);
    }
    eval_expr_str(expr_str, params)
}

fn generate_footprint(shape: &str, width: f32, depth: f32) -> Vec<Vec2> {
    match shape {
        "circle" => {
            // Approximate circle with 16 vertices
            let radius = width / 2.0;
            (0..16)
                .map(|i| {
                    let angle = (i as f32) * std::f32::consts::TAU / 16.0;
                    Vec2::new(angle.cos() * radius, angle.sin() * radius)
                })
                .collect()
        }
        _ => {
            // Rectangle (default)
            vec![
                Vec2::new(0.0, 0.0),
                Vec2::new(width, 0.0),
                Vec2::new(width, depth),
                Vec2::new(0.0, depth),
            ]
        }
    }
}

fn resolve_anchor(anchor: &AnchorDef, params: &HashMap<String, f32>) -> Result<ResolvedAnchor, BlueprintError> {
    let x = eval_expr_str(&anchor.position[0], params)?;
    let y = eval_expr_str(&anchor.position[1], params)?;
    let z = eval_expr_str(&anchor.position[2], params)?;

    let direction = match anchor.direction.as_str() {
        "north" => glam::Vec3::new(0.0, 0.0, -1.0),
        "south" => glam::Vec3::new(0.0, 0.0, 1.0),
        "east" => glam::Vec3::new(1.0, 0.0, 0.0),
        "west" => glam::Vec3::new(-1.0, 0.0, 0.0),
        "up" => glam::Vec3::new(0.0, 1.0, 0.0),
        "down" => glam::Vec3::new(0.0, -1.0, 0.0),
        _ => glam::Vec3::ZERO,
    };

    Ok(ResolvedAnchor {
        name: anchor.name.clone(),
        position: glam::Vec3::new(x, y, z),
        direction,
        tags: anchor.tags.clone(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_blueprint() -> Blueprint {
        let toml_str = r#"
[meta]
id = "test_wall"
name = "Test Wall"
category = "wall"
origin = "constructed"

[parameters]
length = { type = "float", min = 1.0, max = 20.0, default = 5.0 }
height = { type = "float", min = 1.0, max = 5.0, default = 2.5 }

[geometry]
width = "length"
depth = "0.5"
height = "height"

[stats.military]
max_hp = "length * height * 80"
cover_value = "1.0"
blocks_movement = "1"

[[damage_states]]
name = "intact"
threshold = 0.5
"#;
        toml::from_str(toml_str).unwrap()
    }

    #[test]
    fn test_register_and_get() {
        let mut registry = BlueprintRegistry::new();
        let blueprint = create_test_blueprint();

        let id = registry.register(blueprint);

        assert!(registry.get(id).is_some());
        assert!(registry.get_by_name("test_wall").is_some());
    }

    #[test]
    fn test_instantiate() {
        let mut registry = BlueprintRegistry::new();
        let blueprint = create_test_blueprint();
        let id = registry.register(blueprint);

        let mut params = HashMap::new();
        params.insert("length".to_string(), 10.0);
        params.insert("height".to_string(), 3.0);

        let instance = registry
            .instantiate(id, params, Vec2::new(5.0, 10.0), 0.0, PlacedBy::TerrainGen, None)
            .unwrap();

        assert_eq!(instance.geometry.width, 10.0);
        assert_eq!(instance.geometry.height, 3.0);
        assert_eq!(instance.max_hp, 2400.0); // 10 * 3 * 80
        assert!(instance.military.blocks_movement);
    }

    #[test]
    fn test_validation() {
        let mut registry = BlueprintRegistry::new();
        let blueprint = create_test_blueprint();
        let id = registry.register(blueprint);

        // Valid params
        let mut params = HashMap::new();
        params.insert("length".to_string(), 5.0);
        params.insert("height".to_string(), 2.0);
        assert!(registry.validate_params(id, &params).is_ok());

        // Out of range
        params.insert("length".to_string(), 100.0); // max is 20
        assert!(registry.validate_params(id, &params).is_err());
    }

    #[test]
    fn test_defaults() {
        let mut registry = BlueprintRegistry::new();
        let blueprint = create_test_blueprint();
        let id = registry.register(blueprint);

        // Empty params - should use defaults
        let instance = registry
            .instantiate(id, HashMap::new(), Vec2::ZERO, 0.0, PlacedBy::TerrainGen, None)
            .unwrap();

        assert_eq!(instance.geometry.width, 5.0); // default length
        assert_eq!(instance.geometry.height, 2.5); // default height
    }
}
```

**Step 2: Update mod.rs**

Update `src/blueprints/mod.rs`:

```rust
pub mod expression;
pub mod schema;
pub mod instance;
pub mod registry;

pub use expression::{Expr, BinOp, UnaryOp, EvalError};
pub use schema::*;
pub use instance::*;
pub use registry::*;
```

**Step 3: Run tests**

Run: `cargo test blueprints::registry::tests`
Expected: All tests pass

**Step 4: Commit**

```bash
git add src/blueprints/registry.rs src/blueprints/mod.rs
git commit -m "feat: add BlueprintRegistry with loading and instantiation"
```

---

## Task 7: Damage Resolution

**Files:**
- Create: `src/blueprints/damage.rs`
- Modify: `src/blueprints/mod.rs`

**Step 1: Create damage resolution module**

Create `src/blueprints/damage.rs`:

```rust
use glam::Vec2;
use super::instance::{BlueprintInstance, Breach};
use super::schema::{Blueprint, DamageStateDef};

/// Result of applying damage
#[derive(Debug, Clone)]
pub struct DamageResult {
    pub new_state: Option<String>,
    pub destroyed: bool,
    pub new_breach: Option<Breach>,
    pub rubble_produced: bool,
}

/// Apply damage to an instance and update its state
pub fn apply_damage(
    instance: &mut BlueprintInstance,
    amount: f32,
    impact_point: Vec2,
    blueprint: &Blueprint,
) -> DamageResult {
    let old_state = instance.damage_state.clone();
    let was_destroyed = instance.apply_damage(amount);

    // Find new damage state based on HP ratio
    let hp_ratio = instance.hp_ratio();
    let new_state = find_damage_state(&blueprint.damage_states, hp_ratio);

    let mut result = DamageResult {
        new_state: None,
        destroyed: was_destroyed,
        new_breach: None,
        rubble_produced: false,
    };

    if let Some(state) = new_state {
        if state.name != old_state {
            // State changed
            instance.damage_state = state.name.clone();
            result.new_state = Some(state.name.clone());

            // Apply property overrides
            instance.apply_overrides(&state.overrides);

            // Handle breach
            if state.creates_breach {
                let breach = Breach {
                    position: impact_point,
                    width: 2.0, // Default breach width
                };
                instance.breaches.push(breach.clone());
                result.new_breach = Some(breach);
            }

            // Handle rubble
            if state.produces_rubble {
                result.rubble_produced = true;
            }
        }
    }

    result
}

/// Find the appropriate damage state for a given HP ratio
/// States are sorted by threshold descending; first state where hp_ratio <= threshold wins
fn find_damage_state<'a>(
    states: &'a [DamageStateDef],
    hp_ratio: f32,
) -> Option<&'a DamageStateDef> {
    // Sort states by threshold descending
    let mut sorted: Vec<&DamageStateDef> = states.iter().collect();
    sorted.sort_by(|a, b| b.threshold.partial_cmp(&a.threshold).unwrap());

    // Find first state where hp_ratio <= threshold
    sorted.into_iter().find(|s| hp_ratio <= s.threshold)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::blueprints::*;
    use std::collections::HashMap;

    fn create_test_blueprint() -> Blueprint {
        let toml_str = r#"
[meta]
id = "test_wall"
name = "Test Wall"
category = "wall"

[parameters]
length = { type = "float", min = 1.0, max = 20.0, default = 5.0 }
height = { type = "float", min = 1.0, max = 5.0, default = 2.5 }

[geometry]
width = "length"
depth = "0.5"
height = "height"

[stats.military]
max_hp = "100"
cover_value = "1.0"

[[damage_states]]
name = "intact"
threshold = 1.0

[[damage_states]]
name = "damaged"
threshold = 0.5

[[damage_states]]
name = "breached"
threshold = 0.25
creates_breach = true
produces_rubble = true
overrides = { cover_value = 0.5, blocks_movement = false }

[[damage_states]]
name = "destroyed"
threshold = 0.0
"#;
        toml::from_str(toml_str).unwrap()
    }

    #[test]
    fn test_find_damage_state() {
        let blueprint = create_test_blueprint();

        let state = find_damage_state(&blueprint.damage_states, 1.0);
        assert_eq!(state.unwrap().name, "intact");

        let state = find_damage_state(&blueprint.damage_states, 0.6);
        assert_eq!(state.unwrap().name, "damaged");

        let state = find_damage_state(&blueprint.damage_states, 0.3);
        assert_eq!(state.unwrap().name, "damaged");

        let state = find_damage_state(&blueprint.damage_states, 0.2);
        assert_eq!(state.unwrap().name, "breached");

        let state = find_damage_state(&blueprint.damage_states, 0.0);
        assert_eq!(state.unwrap().name, "destroyed");
    }

    #[test]
    fn test_apply_damage_state_transition() {
        let mut registry = BlueprintRegistry::new();
        let blueprint = create_test_blueprint();
        let id = registry.register(blueprint.clone());

        let mut instance = registry
            .instantiate(id, HashMap::new(), Vec2::ZERO, 0.0, PlacedBy::TerrainGen, None)
            .unwrap();

        // Start at full HP
        assert_eq!(instance.damage_state, "intact");
        assert_eq!(instance.current_hp, 100.0);

        // Damage to 60% HP
        let result = apply_damage(&mut instance, 40.0, Vec2::ZERO, &blueprint);
        assert_eq!(instance.damage_state, "damaged");
        assert!(result.new_state.is_some());
        assert!(!result.destroyed);

        // Damage to 20% HP - should create breach
        let result = apply_damage(&mut instance, 40.0, Vec2::new(1.0, 0.0), &blueprint);
        assert_eq!(instance.damage_state, "breached");
        assert!(result.new_breach.is_some());
        assert!(result.rubble_produced);
        assert!(!instance.military.blocks_movement); // Override applied

        // Destroy
        let result = apply_damage(&mut instance, 100.0, Vec2::ZERO, &blueprint);
        assert!(result.destroyed);
        assert_eq!(instance.current_hp, 0.0);
    }
}
```

**Step 2: Update mod.rs**

Update `src/blueprints/mod.rs`:

```rust
pub mod expression;
pub mod schema;
pub mod instance;
pub mod registry;
pub mod damage;

pub use expression::{Expr, BinOp, UnaryOp, EvalError};
pub use schema::*;
pub use instance::*;
pub use registry::*;
pub use damage::{apply_damage, DamageResult};
```

**Step 3: Run tests**

Run: `cargo test blueprints::damage::tests`
Expected: All tests pass

**Step 4: Commit**

```bash
git add src/blueprints/damage.rs src/blueprints/mod.rs
git commit -m "feat: add damage resolution with state transitions"
```

---

## Task 8: Construction System

**Files:**
- Create: `src/blueprints/construction.rs`
- Modify: `src/blueprints/mod.rs`

**Step 1: Create construction system**

Create `src/blueprints/construction.rs`:

```rust
use std::collections::HashMap;
use super::instance::BlueprintInstance;
use super::schema::{Blueprint, ConstructionStageDef};
use super::expression::Expr;

/// Progress construction on an instance
/// Returns true if construction completed this tick
pub fn apply_work(
    instance: &mut BlueprintInstance,
    work_amount: f32,
    blueprint: &Blueprint,
) -> bool {
    if instance.is_complete() {
        return false;
    }

    let construction = match &blueprint.construction {
        Some(c) => c,
        None => {
            // No construction defined, instantly complete
            instance.construction_progress = 1.0;
            instance.construction_stage = None;
            return true;
        }
    };

    // Calculate total work required
    let total_work = match Expr::parse(&construction.base_time) {
        Ok(expr) => expr.evaluate(&instance.parameters).unwrap_or(100.0),
        Err(_) => 100.0,
    };

    // Apply work
    let progress_delta = work_amount / total_work;
    instance.construction_progress = (instance.construction_progress + progress_delta).min(1.0);

    // Find current stage
    let new_stage = find_construction_stage(&construction.stages, instance.construction_progress);
    if let Some(stage) = new_stage {
        if instance.construction_stage.as_ref() != Some(&stage.id) {
            instance.construction_stage = Some(stage.id.clone());
            // Apply stage property overrides
            instance.apply_overrides(&stage.overrides);
            // Scale geometry height
            instance.geometry.height *= stage.height_multiplier;
        }
    }

    instance.is_complete()
}

/// Get materials required for construction
pub fn get_required_materials(
    blueprint: &Blueprint,
    params: &HashMap<String, f32>,
) -> HashMap<String, u32> {
    let mut materials = HashMap::new();

    if let Some(construction) = &blueprint.construction {
        for (material, expr_str) in &construction.cost {
            if let Ok(expr) = Expr::parse(expr_str) {
                if let Ok(amount) = expr.evaluate(params) {
                    materials.insert(material.clone(), amount.ceil() as u32);
                }
            }
        }
    }

    materials
}

/// Get labor cap (max workers) for construction
pub fn get_labor_cap(blueprint: &Blueprint, params: &HashMap<String, f32>) -> u32 {
    if let Some(construction) = &blueprint.construction {
        if !construction.labor_cap.is_empty() {
            if let Ok(expr) = Expr::parse(&construction.labor_cap) {
                if let Ok(cap) = expr.evaluate(params) {
                    return cap.ceil() as u32;
                }
            }
        }
    }
    1 // Default to 1 worker
}

fn find_construction_stage<'a>(
    stages: &'a [ConstructionStageDef],
    progress: f32,
) -> Option<&'a ConstructionStageDef> {
    // Sort stages by progress_threshold ascending
    let mut sorted: Vec<&ConstructionStageDef> = stages.iter().collect();
    sorted.sort_by(|a, b| a.progress_threshold.partial_cmp(&b.progress_threshold).unwrap());

    // Find highest stage where progress >= threshold
    sorted.into_iter().rev().find(|s| progress >= s.progress_threshold)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::blueprints::*;
    use glam::Vec2;

    fn create_test_blueprint() -> Blueprint {
        let toml_str = r#"
[meta]
id = "test_wall"
name = "Test Wall"
category = "wall"
origin = "constructed"

[parameters]
length = { type = "float", min = 1.0, max = 20.0, default = 5.0 }

[geometry]
width = "length"
depth = "0.5"
height = "3.0"

[stats.military]
max_hp = "100"

[construction]
base_time = "100"  # 100 work units
labor_cap = "ceil(length / 2)"

[construction.cost]
stone = "length * 10"
wood = "length * 2"

[[construction.stages]]
id = "foundation"
progress_threshold = 0.0
height_multiplier = 0.2
visual_state = "foundation"
overrides = { blocks_movement = false }

[[construction.stages]]
id = "half"
progress_threshold = 0.5
height_multiplier = 0.6
visual_state = "half"
overrides = { blocks_movement = true }

[[construction.stages]]
id = "complete"
progress_threshold = 1.0
height_multiplier = 1.0
visual_state = "complete"
"#;
        toml::from_str(toml_str).unwrap()
    }

    #[test]
    fn test_get_required_materials() {
        let blueprint = create_test_blueprint();
        let mut params = HashMap::new();
        params.insert("length".to_string(), 10.0);

        let materials = get_required_materials(&blueprint, &params);

        assert_eq!(materials.get("stone"), Some(&100)); // 10 * 10
        assert_eq!(materials.get("wood"), Some(&20)); // 10 * 2
    }

    #[test]
    fn test_get_labor_cap() {
        let blueprint = create_test_blueprint();
        let mut params = HashMap::new();
        params.insert("length".to_string(), 10.0);

        let cap = get_labor_cap(&blueprint, &params);
        assert_eq!(cap, 5); // ceil(10 / 2)
    }

    #[test]
    fn test_apply_work() {
        let mut registry = BlueprintRegistry::new();
        let blueprint = create_test_blueprint();
        let id = registry.register(blueprint.clone());

        let mut params = HashMap::new();
        params.insert("length".to_string(), 5.0);

        let mut instance = registry
            .instantiate(id, params, Vec2::ZERO, 0.0, PlacedBy::Gameplay { tick: 0 }, Some(1))
            .unwrap();

        // Start at 0 progress
        assert_eq!(instance.construction_progress, 0.0);
        assert_eq!(instance.construction_stage, Some("foundation".to_string()));
        assert!(!instance.is_complete());

        // Apply 50 work (base_time is 100)
        let completed = apply_work(&mut instance, 50.0, &blueprint);
        assert!(!completed);
        assert_eq!(instance.construction_progress, 0.5);
        assert_eq!(instance.construction_stage, Some("half".to_string()));

        // Apply remaining work
        let completed = apply_work(&mut instance, 50.0, &blueprint);
        assert!(completed);
        assert_eq!(instance.construction_progress, 1.0);
        assert!(instance.is_complete());
    }
}
```

**Step 2: Update mod.rs**

Update `src/blueprints/mod.rs`:

```rust
pub mod expression;
pub mod schema;
pub mod instance;
pub mod registry;
pub mod damage;
pub mod construction;

pub use expression::{Expr, BinOp, UnaryOp, EvalError};
pub use schema::*;
pub use instance::*;
pub use registry::*;
pub use damage::{apply_damage, DamageResult};
pub use construction::{apply_work, get_required_materials, get_labor_cap};
```

**Step 3: Run tests**

Run: `cargo test blueprints::construction::tests`
Expected: All tests pass

**Step 4: Commit**

```bash
git add src/blueprints/construction.rs src/blueprints/mod.rs
git commit -m "feat: add construction system with stages and material costs"
```

---

## Task 9: Create Example Blueprint Files

**Files:**
- Create: `data/blueprints/constructed/stone_wall.toml`
- Create: `data/blueprints/natural/oak_tree.toml`
- Create: `data/blueprints/natural/rock_outcrop.toml`

**Step 1: Create directory structure**

Run: `mkdir -p data/blueprints/constructed data/blueprints/natural`

**Step 2: Create stone_wall.toml**

```toml
[meta]
id = "stone_wall"
name = "Stone Wall"
category = "wall"
origin = "constructed"
description = "A defensive stone wall segment"

[parameters]
length = { type = "float", min = 2.0, max = 20.0, default = 5.0 }
height = { type = "float", min = 1.0, max = 5.0, default = 2.5 }
thickness = { type = "float", min = 0.4, max = 1.5, default = 0.6 }

[geometry]
width = "length"
depth = "thickness"
height = "height"
shape = "rectangle"

[stats.military]
max_hp = "length * height * 80"
hardness = "10"
cover_value = "height >= 1.5 ? 1.0 : 0.5"
blocks_movement = "1"
blocks_los = "height >= 1.8 ? 1 : 0"
movement_cost = "999"
flammable = "0"

[stats.civilian]
prestige_modifier = "1.0"

[construction]
base_time = "length * height * 20"
labor_cap = "ceil(length / 2)"

[construction.cost]
stone = "ceil(length * height * 5)"
mortar = "ceil(length * height * 1.5)"

[[construction.stages]]
id = "foundation"
progress_threshold = 0.0
height_multiplier = 0.1
visual_state = "foundation"
overrides = { cover_value = 0.0, blocks_movement = false, blocks_los = false }

[[construction.stages]]
id = "half_height"
progress_threshold = 0.33
height_multiplier = 0.5
visual_state = "half_built"
overrides = { cover_value = 0.5, blocks_movement = true, blocks_los = false }

[[construction.stages]]
id = "near_complete"
progress_threshold = 0.66
height_multiplier = 0.85
visual_state = "near_complete"

[[construction.stages]]
id = "complete"
progress_threshold = 1.0
height_multiplier = 1.0
visual_state = "complete"

[[anchors]]
name = "west"
position = ["0", "0", "thickness / 2"]
direction = "west"
tags = ["wall_joint"]

[[anchors]]
name = "east"
position = ["length", "0", "thickness / 2"]
direction = "east"
tags = ["wall_joint"]

[[anchors]]
name = "top"
position = ["length / 2", "height", "thickness / 2"]
direction = "up"
tags = ["walkway"]

[[damage_states]]
name = "intact"
threshold = 1.0
visual_overlay = ""

[[damage_states]]
name = "damaged"
threshold = 0.5
visual_overlay = "cracks"

[[damage_states]]
name = "breached"
threshold = 0.25
visual_overlay = "rubble"
creates_breach = true
produces_rubble = true
overrides = { cover_value = 0.5, blocks_movement = false, blocks_los = false }

[[damage_states]]
name = "collapsed"
threshold = 0.0
visual_overlay = "ruins"
produces_rubble = true
overrides = { cover_value = 0.1, blocks_movement = false, blocks_los = false }

[[constraints]]
description = "Minimum structural length"
expression = "length >= 2"
error_message = "Wall must be at least 2m long"

[[constraints]]
description = "Height/length stability"
expression = "height <= length * 2"
error_message = "Wall too tall for its length"

[[constraints]]
description = "Thickness/height stability"
expression = "thickness >= height * 0.15"
error_message = "Wall too thin for its height"
```

**Step 3: Create oak_tree.toml**

```toml
[meta]
id = "oak_tree"
name = "Oak Tree"
category = "tree"
origin = "natural"
description = "A mature oak tree providing cover and blocking line of sight"

[parameters]
height = { type = "float", min = 8.0, max = 25.0, default = 15.0 }
canopy_radius = { type = "float", min = 3.0, max = 10.0, default = 5.0 }
trunk_radius = { type = "float", min = 0.3, max = 1.5, default = 0.5 }

[geometry]
width = "trunk_radius * 2"
depth = "trunk_radius * 2"
height = "height"
shape = "circle"

[stats.military]
max_hp = "height * 20"
cover_value = "0.5"
blocks_movement = "1"
blocks_los = "1"
movement_cost = "999"
flammable = "1"

[stats.civilian]
prestige_modifier = "1.1"

[[damage_states]]
name = "healthy"
threshold = 1.0

[[damage_states]]
name = "damaged"
threshold = 0.5
visual_overlay = "damaged_bark"

[[damage_states]]
name = "fallen"
threshold = 0.0
produces_rubble = true
overrides = { blocks_movement = false, blocks_los = false, cover_value = 0.3 }

[[constraints]]
description = "Canopy larger than trunk"
expression = "canopy_radius > trunk_radius"
error_message = "Canopy radius must exceed trunk radius"
```

**Step 4: Create rock_outcrop.toml**

```toml
[meta]
id = "rock_outcrop"
name = "Rock Outcrop"
category = "rock"
origin = "natural"
description = "A natural rock formation providing full cover"

[parameters]
width = { type = "float", min = 1.0, max = 15.0, default = 4.0 }
depth = { type = "float", min = 1.0, max = 10.0, default = 3.0 }
height = { type = "float", min = 0.5, max = 6.0, default = 2.0 }

[geometry]
width = "width"
depth = "depth"
height = "height"
shape = "rectangle"

[stats.military]
max_hp = "999999"
cover_value = "height >= 1.5 ? 1.0 : 0.5"
blocks_movement = "1"
blocks_los = "height >= 1.8 ? 1 : 0"
movement_cost = "999"
flammable = "0"
elevation = "height"

[[damage_states]]
name = "intact"
threshold = 0.0
```

**Step 5: Verify files can be loaded**

Add integration test to `src/blueprints/registry.rs`:

```rust
    #[test]
    fn test_load_example_blueprints() {
        use std::path::Path;

        let mut registry = BlueprintRegistry::new();

        // This test only runs if data directory exists
        let data_path = Path::new("data/blueprints");
        if data_path.exists() {
            let loaded = registry.load_directory(data_path).unwrap();
            assert!(!loaded.is_empty(), "Should load at least one blueprint");

            // Check specific blueprints
            if let Some(wall) = registry.get_by_name("stone_wall") {
                assert_eq!(wall.meta.origin, OriginType::Constructed);
            }
            if let Some(tree) = registry.get_by_name("oak_tree") {
                assert_eq!(tree.meta.origin, OriginType::Natural);
            }
        }
    }
```

**Step 6: Run test**

Run: `cargo test test_load_example_blueprints`
Expected: Test passes (or skips if data dir doesn't exist)

**Step 7: Commit**

```bash
git add data/blueprints/ src/blueprints/registry.rs
git commit -m "feat: add example blueprint TOML files"
```

---

## Task 10: Integration Test

**Files:**
- Create: `tests/blueprint_integration.rs`

**Step 1: Write comprehensive integration test**

Create `tests/blueprint_integration.rs`:

```rust
use arc_citadel::blueprints::*;
use glam::Vec2;
use std::collections::HashMap;
use std::path::Path;

#[test]
fn test_full_blueprint_lifecycle() {
    let mut registry = BlueprintRegistry::new();

    // Load blueprints from data directory
    let data_path = Path::new("data/blueprints");
    if data_path.exists() {
        registry.load_directory(data_path).expect("Failed to load blueprints");
    }

    // If no files exist, register test blueprints manually
    if registry.get_by_name("stone_wall").is_none() {
        let wall_toml = r#"
[meta]
id = "stone_wall"
name = "Stone Wall"
category = "wall"
origin = "constructed"

[parameters]
length = { type = "float", min = 2.0, max = 20.0, default = 5.0 }
height = { type = "float", min = 1.0, max = 5.0, default = 2.5 }

[geometry]
width = "length"
depth = "0.6"
height = "height"

[stats.military]
max_hp = "length * height * 80"
blocks_movement = "1"

[construction]
base_time = "100"

[construction.cost]
stone = "length * 10"

[[construction.stages]]
id = "foundation"
progress_threshold = 0.0
height_multiplier = 0.2
visual_state = "foundation"

[[construction.stages]]
id = "complete"
progress_threshold = 1.0
height_multiplier = 1.0
visual_state = "complete"

[[damage_states]]
name = "intact"
threshold = 1.0

[[damage_states]]
name = "breached"
threshold = 0.25
creates_breach = true
overrides = { blocks_movement = false }
"#;
        let blueprint: Blueprint = toml::from_str(wall_toml).unwrap();
        registry.register(blueprint);
    }

    if registry.get_by_name("oak_tree").is_none() {
        let tree_toml = r#"
[meta]
id = "oak_tree"
name = "Oak Tree"
category = "tree"
origin = "natural"

[parameters]
height = { type = "float", min = 8.0, max = 25.0, default = 15.0 }

[geometry]
width = "1.0"
depth = "1.0"
height = "height"

[stats.military]
max_hp = "height * 20"
flammable = "1"
blocks_movement = "1"

[[damage_states]]
name = "healthy"
threshold = 1.0

[[damage_states]]
name = "fallen"
threshold = 0.0
overrides = { blocks_movement = false }
"#;
        let blueprint: Blueprint = toml::from_str(tree_toml).unwrap();
        registry.register(blueprint);
    }

    // Test natural feature instantiation
    let tree_id = registry.id_by_name("oak_tree").expect("oak_tree not found");
    let tree_blueprint = registry.get(tree_id).unwrap().clone();

    let mut tree_params = HashMap::new();
    tree_params.insert("height".to_string(), 20.0);

    let tree = registry
        .instantiate(
            tree_id,
            tree_params,
            Vec2::new(10.0, 10.0),
            0.0,
            PlacedBy::TerrainGen,
            None,
        )
        .expect("Failed to instantiate tree");

    assert!(tree.is_complete(), "Natural features should spawn complete");
    assert_eq!(tree.max_hp, 400.0, "HP should be height * 20");
    assert!(tree.military.flammable, "Tree should be flammable");
    assert!(tree.military.blocks_movement, "Tree should block movement");

    // Test constructed feature instantiation
    let wall_id = registry.id_by_name("stone_wall").expect("stone_wall not found");
    let wall_blueprint = registry.get(wall_id).unwrap().clone();

    let mut wall_params = HashMap::new();
    wall_params.insert("length".to_string(), 10.0);
    wall_params.insert("height".to_string(), 3.0);

    let mut wall = registry
        .instantiate(
            wall_id,
            wall_params.clone(),
            Vec2::new(20.0, 20.0),
            0.0,
            PlacedBy::Gameplay { tick: 100 },
            Some(1),
        )
        .expect("Failed to instantiate wall");

    assert!(!wall.is_complete(), "Constructed features should start incomplete");
    assert_eq!(wall.construction_progress, 0.0);
    assert_eq!(wall.max_hp, 2400.0, "HP should be length * height * 80");

    // Test construction
    let materials = get_required_materials(&wall_blueprint, &wall_params);
    assert_eq!(materials.get("stone"), Some(&100), "Stone cost should be length * 10");

    // Apply work until complete
    while !wall.is_complete() {
        apply_work(&mut wall, 25.0, &wall_blueprint);
    }
    assert!(wall.is_complete());
    assert_eq!(wall.construction_progress, 1.0);

    // Test damage
    let result = apply_damage(&mut wall, 2000.0, Vec2::ZERO, &wall_blueprint);
    assert!(result.new_state.is_some());
    assert!(result.new_breach.is_some(), "Should create breach at 25% HP");
    assert!(!wall.military.blocks_movement, "Breached wall shouldn't block movement");

    // Test tree damage (fire)
    let mut tree = registry
        .instantiate(
            tree_id,
            HashMap::from([("height".to_string(), 15.0)]),
            Vec2::ZERO,
            0.0,
            PlacedBy::TerrainGen,
            None,
        )
        .unwrap();

    assert!(tree.military.flammable);
    apply_damage(&mut tree, 500.0, Vec2::ZERO, &tree_blueprint);
    assert_eq!(tree.damage_state, "fallen");
    assert!(!tree.military.blocks_movement, "Fallen tree shouldn't block movement");
}

#[test]
fn test_unified_spatial_concept() {
    // This test demonstrates that natural and constructed features
    // can be queried the same way (conceptually - actual spatial index
    // integration is out of scope for this MVP)

    let mut registry = BlueprintRegistry::new();

    // Register both types
    let tree_toml = r#"
[meta]
id = "tree"
name = "Tree"
category = "tree"
origin = "natural"

[parameters]
height = { type = "float", min = 5.0, max = 20.0, default = 10.0 }

[geometry]
width = "1.0"
depth = "1.0"
height = "height"

[stats.military]
max_hp = "100"
blocks_los = "1"
"#;

    let wall_toml = r#"
[meta]
id = "wall"
name = "Wall"
category = "wall"
origin = "constructed"

[parameters]
length = { type = "float", min = 1.0, max = 20.0, default = 5.0 }

[geometry]
width = "length"
depth = "0.5"
height = "2.0"

[stats.military]
max_hp = "200"
blocks_los = "1"
"#;

    registry.register(toml::from_str(tree_toml).unwrap());
    registry.register(toml::from_str(wall_toml).unwrap());

    let tree_id = registry.id_by_name("tree").unwrap();
    let wall_id = registry.id_by_name("wall").unwrap();

    // Both can be instantiated through the same API
    let tree = registry
        .instantiate(tree_id, HashMap::new(), Vec2::new(0.0, 0.0), 0.0, PlacedBy::TerrainGen, None)
        .unwrap();

    let wall = registry
        .instantiate(wall_id, HashMap::new(), Vec2::new(5.0, 0.0), 0.0, PlacedBy::Gameplay { tick: 0 }, Some(1))
        .unwrap();

    // Both have the same interface for LOS queries
    assert!(tree.military.blocks_los);
    assert!(wall.military.blocks_los);

    // Both have geometry that can be used for spatial indexing
    assert!(!tree.geometry.footprint.is_empty());
    assert!(!wall.geometry.footprint.is_empty());

    // Both track who placed them
    assert!(matches!(tree.placed_by, PlacedBy::TerrainGen));
    assert!(matches!(wall.placed_by, PlacedBy::Gameplay { .. }));
}
```

**Step 2: Run integration tests**

Run: `cargo test --test blueprint_integration`
Expected: All tests pass

**Step 3: Commit**

```bash
git add tests/blueprint_integration.rs
git commit -m "test: add comprehensive blueprint integration tests"
```

---

## Task 11: Final Verification

**Step 1: Run all tests**

Run: `cargo test`
Expected: All tests pass

**Step 2: Check for warnings**

Run: `cargo clippy`
Expected: No errors (warnings are OK for MVP)

**Step 3: Verify build**

Run: `cargo build --release`
Expected: Successful compilation

**Step 4: Final commit**

```bash
git add -A
git commit -m "feat: complete Unified Parameterized Blueprint System MVP

Implemented:
- Expression parser with nom (arithmetic, comparisons, conditionals, functions)
- TOML-based blueprint schema with serde
- BlueprintRegistry for loading and instantiation
- BlueprintInstance with cached evaluated properties
- Construction system with stages and material costs
- Damage system with state transitions and breaches
- PlacedBy tracking for provenance (TerrainGen, HistorySim, Gameplay)
- Example blueprints: stone_wall, oak_tree, rock_outcrop

Natural features spawn complete; constructed features require work.
Same ComponentInstance type for all, same spatial/LOS/damage queries."
```

---

## Summary

### Files Created
- `src/blueprints/mod.rs` - Module exports
- `src/blueprints/expression.rs` - nom-based expression parser
- `src/blueprints/schema.rs` - TOML schema types with serde
- `src/blueprints/instance.rs` - BlueprintInstance runtime type
- `src/blueprints/registry.rs` - BlueprintRegistry with loading/instantiation
- `src/blueprints/damage.rs` - Damage resolution system
- `src/blueprints/construction.rs` - Construction system
- `data/blueprints/constructed/stone_wall.toml` - Example wall blueprint
- `data/blueprints/natural/oak_tree.toml` - Example tree blueprint
- `data/blueprints/natural/rock_outcrop.toml` - Example rock blueprint
- `tests/blueprint_integration.rs` - Integration tests

### Files Modified
- `Cargo.toml` - Add nom dependency
- `src/lib.rs` - Add blueprints module

### Key Decisions
- Expression strings in TOML, compiled at load with nom
- Properties evaluated once at instantiation, cached in instance
- Natural features complete on spawn; constructed need work
- Same BlueprintInstance type for all, unified query interface
- PlacedBy enum tracks provenance (terrain gen, history sim, gameplay)
