use super::parser::*;

type Program<'a> = Vec<Statement<'a>>;
type Ident<'a> = &'a str;

#[derive(Debug)]
enum Statement<'a> {
  LetStmt(Ident<'a>, Expression<'a>),
}

#[derive(Debug)]
enum Expression<'a> {
  LiteralExpr(LiteralValue<'a>),
}

#[derive(Debug)]
enum LiteralValue<'a> {
  Unit,
  Int64(i64),
  Str(&'a str),
  Bool(bool),
}

productions! {

  // Basic types
  ident -> &'a str {
    { name:[r"[a-zA-Z][a-zA-Z0-9_-]*"] } => name
  }

  literal_value -> LiteralValue<'a> {
    { v:[r"[0-9]+"] } => LiteralValue::Int64(v.parse::<i64>().unwrap()),
    { v:[r#""[^"]*""#] } => LiteralValue::Str(&v[1..v.len() - 1]),
    { _:[r"true"] } => LiteralValue::Bool(true),
    { _:[r"false"] }  => LiteralValue::Bool(false),
  }

  statement -> String {
    { _:[r"let"] ident:[ident] _:[r"="] expr:[expression] _:[r";"] } => format!("let {} = {};", ident, expr)
  }

  expression -> String {
    { ident:[ident] } => ident.to_owned(),
    { _:[r"\{"] expr:[expression] _:[r"\}"] } => expr,
  }

}
