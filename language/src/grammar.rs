use crate::parser::end_of_input;

pub type Program = Vec<Statement>;
pub type Ident = String;

#[derive(Clone, Debug)]
pub enum Statement {
  LetStmt(Ident, Expression),
  Assignment(Ident, Expression),
  FunctionDeclStmt(Ident, Vec<Ident>, CodeBlock),
  CodeBlockStmt(CodeBlock),
  IfElseStmt(Expression, CodeBlock, Option<CodeBlock>),
}

#[derive(Clone, Debug)]
pub enum Expression {
  LiteralExpr(LiteralValue),
  BinaryExpr(Box<Expression>, BinaryOp, Box<Expression>),
  IdentDerefExpr(Ident),
  CodeBlockExpr(Box<CodeBlock>),
  IfElseExpr(Box<Expression>, Box<CodeBlock>, Option<Box<CodeBlock>>),
}

#[derive(Clone, Debug)]
pub enum LiteralValue {
  Unit,
  Float64(f64),
  Int64(i64),
  Str(String),
  Bool(bool),
}

#[derive(Clone, Debug)]
pub struct CodeBlock(Vec<Statement>, Expression);

#[derive(Clone, Debug)]
pub enum BinaryOp {
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

productions! {

  // Root program
  program -> Program {
    { s:[0..; global_decl] _:[end_of_input] } => s,
  }

  global_decl -> Statement {
    { function_decl:[function_decl_statement] } => function_decl,
  }

  // Statements
  statement -> Statement {
    { function_decl:[function_decl_statement] } => function_decl,
    { assignment:[assignment_statement] } => assignment,
    { if_else:[if_else_statement] } => if_else,
  }

  assignment_statement -> Statement {
    { _:[r"let"] ident:[ident] _:[r"="] expr:[expression] _:[r";"] }
        => Statement::LetStmt(ident, expr),
    { ident:[ident] _:[r"="] expr:[expression] _:[r";"] }
        => Statement::Assignment(ident, expr),
  }

  function_decl_statement -> Statement {
    { _:[r"fn"] ident:[ident] params:[param_parentheses_group] block:[code_block_expression] }
        => Statement::FunctionDeclStmt(ident, params, block),
  }

  if_else_statement -> Statement {
    { _:[r"if"] condition:[expression] then_block:[code_block_expression] }
        => Statement::IfElseStmt(condition, then_block, None),
    { _:[r"if"] condition:[expression] then_block:[code_block_expression] _:[r"else"] else_block:[code_block_expression] }
        => Statement::IfElseStmt(condition, then_block, Some(else_block)),
  }

  // Expressions
  expression -> Expression {
    { unary:[unary_expression] } => unary,
    { ident:[ident] } => Expression::IdentDerefExpr(ident),
    { if_else:[if_else_expression] } => if_else,
    { block:[code_block_expression] } => Expression::CodeBlockExpr(Box::new(block)),
  }

  if_else_expression -> Expression {
    { _:[r"if"] condition:[expression] then_block:[code_block_expression] }
        => Expression::IfElseExpr(Box::new(condition), Box::new(then_block), None),
    { _:[r"if"] condition:[expression] then_block:[code_block_expression] _:[r"else"] else_block:[code_block_expression] }
        => Expression::IfElseExpr(Box::new(condition), Box::new(then_block), Some(Box::new(else_block))),
  }

  code_block_expression -> CodeBlock {
    { _:[r"\{"] s:[0..; statement] e:[expression] _:[r"\}"] } => CodeBlock(s, e),
    { _:[r"\{"] s:[0..; statement] _:[r"\}"] }
        => CodeBlock(s, Expression::LiteralExpr(LiteralValue::Unit)),
  }

  // Ident and Literals
  ident -> Ident {
    { name:[r"[a-zA-Z][a-zA-Z0-9_-]*"] } => name.to_owned(),
  }

  literal_value -> LiteralValue {
    { v:[r"[0-9]+\.[0-9]+"] } => LiteralValue::Float64(v.parse::<f64>().unwrap()),
    { v:[r"[0-9]+"] } => LiteralValue::Int64(v.parse::<i64>().unwrap()),
    { v:[r#""[^"]*""#] } => LiteralValue::Str(v[1..v.len() - 1].to_owned()),
    { _:[r"true"] } => LiteralValue::Bool(true),
    { _:[r"false"] }  => LiteralValue::Bool(false),
  }

  // Unary (precedence climbing)
  unary_expression -> Expression {
    { s:[or_expression] } => s,
  }

  or_expression -> Expression {
    { l:[and_expression] _:[r"\|\|"] r:[or_expression] }
        => Expression::BinaryExpr(Box::new(l), BinaryOp::Or, Box::new(r)),
    { p:[and_expression] } => p,
  }

  and_expression -> Expression {
    { l:[eq_expression] _:[r"&&"] r:[and_expression] }
        => Expression::BinaryExpr(Box::new(l), BinaryOp::And, Box::new(r)),
    { p:[eq_expression] } => p,
  }

  eq_expression -> Expression {
    { l:[cmp_expression] _:[r"=="] r:[eq_expression] }
        => Expression::BinaryExpr(Box::new(l), BinaryOp::Eql, Box::new(r)),
    { l:[cmp_expression] _:[r"!="] r:[eq_expression] }
        => Expression::BinaryExpr(Box::new(l), BinaryOp::Neq, Box::new(r)),
    { p:[cmp_expression] } => p,
  }

  cmp_expression -> Expression {
    { l:[sum_expression] _:[r">"] r:[cmp_expression] }
        => Expression::BinaryExpr(Box::new(l), BinaryOp::Gt, Box::new(r)),
    { l:[sum_expression] _:[r"<"] r:[cmp_expression] }
        => Expression::BinaryExpr(Box::new(l), BinaryOp::Lt, Box::new(r)),
    { l:[sum_expression] _:[r">="] r:[cmp_expression] }
        => Expression::BinaryExpr(Box::new(l), BinaryOp::Ge, Box::new(r)),
    { l:[sum_expression] _:[r"<="] r:[cmp_expression] }
        => Expression::BinaryExpr(Box::new(l), BinaryOp::Le, Box::new(r)),
    { p:[sum_expression] } => p,
  }

  sum_expression -> Expression {
    { l:[product_expression] _:[r"\+"] r:[sum_expression] }
        => Expression::BinaryExpr(Box::new(l), BinaryOp::Plus, Box::new(r)),
    { l:[product_expression] _:[r"-"] r:[sum_expression] }
        => Expression::BinaryExpr(Box::new(l), BinaryOp::Minus, Box::new(r)),
    { p:[product_expression] } => p,
  }

  product_expression -> Expression {
    { l:[unary_atom] _:[r"\*"] r:[product_expression] }
        => Expression::BinaryExpr(Box::new(l), BinaryOp::Star, Box::new(r)),
    { l:[unary_atom] _:[r"/"] r:[product_expression] }
        => Expression::BinaryExpr(Box::new(l), BinaryOp::Slash, Box::new(r)),
    { a:[unary_atom] } => a,
  }

  unary_atom -> Expression {
    { _:[r"\("] expr:[unary_expression] _:[r"\)"] } => expr,
    { v:[literal_value] } => Expression::LiteralExpr(v),
    { ident:[ident] } => Expression::IdentDerefExpr(ident),
  }

  // Parentheses group
  param_parentheses_group -> Vec<Ident> {
    { _:[r"\("] e1:[0..; param_in_parentheses] e2:[ident] _:[0..1; r","] _:[r"\)"] } => {
      let mut vec = e1;
      vec.push(e2);
      vec
    },
    { _:[r"\("] _:[r"\)"] } => vec![],
  }

  param_in_parentheses -> Ident {
    { ident:[ident] _:[r","] } => ident,
  }

  args_parentheses_group -> Vec<Expression> {
    { _:[r"\("] e1:[0..; arg_in_parentheses] e2:[expression] _:[0..1; r","] _:[r"\)"] } => {
      let mut vec = e1;
      vec.push(e2);
      vec
    },
    { _:[r"\("] _:[r"\)"] } => vec![],
  }

  arg_in_parentheses -> Expression {
    { expr:[expression] _:[r","] } => expr,
  }

}
