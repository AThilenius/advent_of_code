use std::cmp::{Eq, PartialEq};
use std::fmt;
use std::hash::{Hash, Hasher};

/**
 * A reference back to the original source that produces this AST node.
 */
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct SourceRef {
  pub left: u64,
  pub right: u64,
}

impl SourceRef {
  pub fn new(l: usize, r: usize) -> SourceRef {
    SourceRef {
      left: l as u64,
      right: r as u64,
    }
  }
}

impl fmt::Debug for SourceRef {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    // Suppress debug output
    write!(f, "")
  }
}

/**
 * An Ident.
 */
#[derive(Clone, Eq)]
pub struct Identifier {
  pub name: String,
  pub source_ref: SourceRef,
}

impl fmt::Debug for Identifier {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "Identifier \"{}\"", self.name)
  }
}

impl PartialEq for Identifier {
  fn eq(&self, other: &Identifier) -> bool {
    self.name == other.name
  }
}

impl Hash for Identifier {
  fn hash<H: Hasher>(&self, state: &mut H) {
    self.name.hash(state);
  }
}

/**
 * A block is a vector or statements followed by a return expression. Blocks
 * have a list of 'free variables' that can be bound lexicographically or
 * dynamically (the latter case being used for function arguments).
 *
 *
 * A block can be found in many forms:
 *
 * A simple expression only block without free variables;
 * > { 42 }
 *
 * A closed-over block with one free variable 'foo':
 * > { foo + 42 }
 *
 * A closed-over block with Statements and a ReturnExpression:
 * > { let bar = foo + 42; bar }
 *
 * A block can also be bound to a function. It acts just like any other block
 * with the exception that the input parameters are dynamically scoped. In other
 * words:
 * > fn (arg) { foo + arg + 42 }
 * produces a block with the free variables one and two. These free variables
 * do not need to be lexicographically closed over though.
 *
 * A block owens it's enclosed statements and return expression.
 */
#[derive(Clone, Debug)]
pub struct Block {
  pub statements: Vec<Statement>,
  pub return_expression: Expression,
  pub source_ref: SourceRef,
}

impl Block {
  pub fn new(stmt: Vec<Statement>, ret: Expression, src: SourceRef) -> Block {
    Block {
      statements: stmt,
      return_expression: ret,
      source_ref: src,
    }
  }
}

/**
 * A statement is always owned by a Block. It is anything with a ";" after it.
 */
#[derive(Clone, Debug)]
pub enum Statement {
  // A mutable let statement.
  // Ex: let foo = 42;
  LetStmt(Identifier, Expression),

  // An assignment to an *already bound* variable.
  // Ex: foo = 24;
  AssignmentStmt(Identifier, Expression),

  // An unused expression evaluation. This is for transitive effects of the
  // expression and does not directly mutate the current scope.
  // Ex: { 42 + 24 }; returns_something_that_we_are_ignoring();
  UnusedExprEvalStmt(Expression),

  // A function declaration statement. This is a special case, it is the only
  // statement that doesn't need to in a ";".
  // Ex: fn name (arg1, arg2) { }  fn name { }
  FunctionDeclarationStmt(Identifier, Vec<Identifier>, Box<Block>),
}

/**
 * An expression is always ultimately owned by a Block, but can be nested down
 * in a long expression tree.
 */
#[derive(Clone, Debug)]
pub enum Expression {
  // A literal value.
  // Ex: 42, "hello, world!", false
  LiteralExpr(LiteralValue),

  // A recursive binary operation.
  // Ex: 1 + 2, true && false, 42 > 24
  BinExpr(Box<Expression>, BinOp, Box<Expression>),

  // A child block used as an expression. Note that is directly owned as this
  // forms part os a tree-structure.
  // Ex: { 42 }
  BlockExpr(Box<Block>),

  // An variable ident ref used as an expression.
  // Ex: foo + 42
  IdentifierDerefExpr(Identifier),

  // The results of evaluating a Block with dynamically scoped params.
  // Ex: returns_42(arg1, arg2)
  FunctionInvokeExpr(Identifier, Vec<Expression>),

  // An if+else (else is require) expression.
  // Es: let foo = if bar { 1 } else { 2 };
  IfElseExpr(Box<Expression>, Box<Block>, Option<Box<Block>>),
}

impl Expression {
  fn get_free_variables(&self) -> Vec<&Identifier> {
    match self {
      // LetStmt(_, ref expression) =>
      _ => vec![],
    }
  }
}

#[derive(Clone, Debug)]
pub enum LiteralValue {
  Unit,
  Int64(i64),
  Str(String),
  Bool(bool),
}

#[derive(Clone, Debug)]
pub enum BinOp {
  Ge,
  Gt,
  Eql,
  Neq,
  Le,
  Lt,

  And,
  Or,

  Minus,
  Plus,
  Slash,
  Star,
  Mod,
}
