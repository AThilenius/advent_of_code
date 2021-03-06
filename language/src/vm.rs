use super::ast::*;
use super::scope::*;
use regex::Regex;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fs;
use std::rc::Rc;

lalrpop_mod!(pub grammar);

pub struct VM {
  parser: grammar::ProgramParser,
  global_scope: Rc<RefCell<Scope>>,
}

impl VM {
  pub fn new() -> VM {
    let mut vm = VM {
      parser: grammar::ProgramParser::new(),
      global_scope: Rc::new(RefCell::new(Scope {
        parent: None,
        locals: HashMap::new(),
      })),
    };
    vm.add_builtin_function("println", |args| {
      for arg in args {
        print!("{} ", arg);
      }
      println!();
      Value::Unit
    });
    vm.add_builtin_function("read_file_to_str", |args| {
      if let Value::Str(ref path) = args[0] {
        return Value::Str(fs::read_to_string(path).expect("Cannot read file"));
      }
      Value::Unit
    });
    vm
  }

  pub fn add_builtin_function(&mut self, name: &str, function: fn(Vec<Value>) -> Value) {
    (*self.global_scope).borrow_mut().bind_variable(
      Identifier {
        name: name.to_owned(),
        source_ref: SourceRef::new(0, 0),
      },
      Value::BuiltInFunction(function),
    );
  }

  pub fn exec(&mut self, source: &str) {
    // Strip out line-level comments (Don't see a better way to do this with LALRPOP)
    let re = Regex::new(r"^\s*//.*$").unwrap();
    let raw_source: String = source.lines().filter(|line| !re.is_match(line)).collect();
    let ast = self.parser.parse(&raw_source).unwrap();
    self.exec_block_on_scope(&mut Rc::clone(&self.global_scope), &ast);
  }

  fn exec_block_on_scope(&self, scope: &mut Rc<RefCell<Scope>>, block: &Block) -> Value {
    for statement in &block.statements {
      match statement {
        Statement::LetStmt(ref identifier, ref expression) => {
          let value = self.eval_expression_on_scope(scope, expression);
          (**scope)
            .borrow_mut()
            .bind_variable(identifier.clone(), value);
        }
        Statement::AssignmentStmt(ref identifier, ref expression) => {
          let value = self.eval_expression_on_scope(scope, expression);
          (**scope).borrow_mut().assign_variable(identifier, value);
        }
        Statement::UnusedExprEvalStmt(ref expression) => {
          self.eval_expression_on_scope(scope, expression);
        }
        Statement::FunctionDeclarationStmt(ref identifier, ref params, ref block) => {
          // Create a new child scope
          (**scope).borrow_mut().bind_variable(
            identifier.clone(),
            Value::Function(Rc::clone(scope), params.clone(), *block.clone()),
          );
        }
      }
    }
    self.eval_expression_on_scope(scope, &block.return_expression)
  }

  fn eval_expression_on_scope(
    &self,
    scope: &mut Rc<RefCell<Scope>>,
    expression: &Expression,
  ) -> Value {
    match expression {
      Expression::LiteralExpr(ref v) => match v {
        LiteralValue::Unit => Value::Unit,
        LiteralValue::Int64(ref i) => Value::Int64(*i),
        LiteralValue::Str(ref i) => Value::Str(i.to_owned()),
        LiteralValue::Bool(ref i) => Value::Bool(*i),
      },
      Expression::BinExpr(lbox, op, rbox) => {
        let l = self.eval_expression_on_scope(scope, lbox);
        let r = self.eval_expression_on_scope(scope, rbox);
        match (op, l, r) {
          // Arithmetic Operations
          (BinOp::Plus, Value::Int64(l), Value::Int64(r)) => Value::Int64(l + r),
          (BinOp::Minus, Value::Int64(l), Value::Int64(r)) => Value::Int64(l - r),
          (BinOp::Star, Value::Int64(l), Value::Int64(r)) => Value::Int64(l * r),
          (BinOp::Slash, Value::Int64(l), Value::Int64(r)) => Value::Int64(l / r),
          (BinOp::Mod, Value::Int64(l), Value::Int64(r)) => Value::Int64(l % r),
          (BinOp::Ge, Value::Int64(l), Value::Int64(r)) => Value::Bool(l >= r),
          (BinOp::Gt, Value::Int64(l), Value::Int64(r)) => Value::Bool(l > r),
          (BinOp::Le, Value::Int64(l), Value::Int64(r)) => Value::Bool(l <= r),
          (BinOp::Lt, Value::Int64(l), Value::Int64(r)) => Value::Bool(l < r),
          (BinOp::Eql, Value::Int64(l), Value::Int64(r)) => Value::Bool(l == r),
          (BinOp::Neq, Value::Int64(l), Value::Int64(r)) => Value::Bool(l != r),

          // Boolean Operations
          (BinOp::Eql, Value::Bool(l), Value::Bool(r)) => Value::Bool(l == r),
          (BinOp::Neq, Value::Bool(l), Value::Bool(r)) => Value::Bool(l != r),
          (BinOp::And, Value::Bool(l), Value::Bool(r)) => Value::Bool(l && r),
          (BinOp::Or, Value::Bool(l), Value::Bool(r)) => Value::Bool(l || r),

          // Unsupported operations
          _ => panic!(format!("Failed to eval binary expression {:?}", expression)),
        }
      }
      Expression::BlockExpr(ref block) => {
        let mut child_scope = push_scope(scope);
        self.exec_block_on_scope(&mut child_scope, block)
      }
      Expression::IdentifierDerefExpr(ref identifier) => {
        (**scope).borrow().get_variable(identifier)
      }
      Expression::FunctionInvokeExpr(ref identifier, ref args) => {
        let mut function = (**scope).borrow().get_variable(identifier);
        match function {
          Value::Function(ref mut closure_scope, ref params, ref block) => {
            let mut function_scope = push_scope(closure_scope);
            // Bind parameters directly into child scope
            if params.len() != args.len() {
              panic!("Mismatched number of arguments");
            }
            for (param, arg_expression) in params.iter().zip(args.iter()) {
              let arg_value = self.eval_expression_on_scope(scope, arg_expression);
              (*function_scope)
                .borrow_mut()
                .close_variable(param, arg_value);
            }
            // Exec the function block
            self.exec_block_on_scope(&mut function_scope, block)
          }
          Value::BuiltInFunction(ref function) => function(
            args
              .iter()
              .map(|arg| self.eval_expression_on_scope(scope, arg))
              .collect(),
          ),
          _ => panic!("Function is not declared {}", identifier.name),
        }
      }
      Expression::IfElseExpr(ref condition, ref then_block, ref opt_else_block) => {
        if let Value::Bool(c) = self.eval_expression_on_scope(scope, condition) {
          let mut child_scope = push_scope(scope);
          if c {
            return self.exec_block_on_scope(&mut child_scope, then_block);
          } else {
            if let Some(else_block) = opt_else_block {
              return self.exec_block_on_scope(&mut child_scope, else_block);
            } else {
              return Value::Unit;
            }
          }
        }
        panic!("Condition is not a boolean expression: {:#?}", condition);
      }
    }
  }
}
