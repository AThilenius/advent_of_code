use super::ast::*;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;

#[derive(Clone)]
pub enum Value {
  // Unit value.
  Unit,
  // Closed-over function.
  Function(Rc<RefCell<Scope>>, Vec<Identifier>, Block),
  // Numeric Values.
  Int64(i64),
  // Other.
  Str(String),
  Bool(bool),
  // Struct
  // ...
  BuiltInFunction(fn(Vec<Value>) -> Value),
}

impl fmt::Debug for Value {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      Value::Unit => write!(f, "()"),
      Value::Function(_, params, _) => write!(f, "Function ({:#?})", params),
      Value::Int64(v) => write!(f, "i{}", v),
      Value::Str(v) => write!(f, "\"{}\"", v),
      Value::Bool(v) => write!(f, "{}", v),
      Value::BuiltInFunction(_) => write!(f, "BuiltInFunction"),
    }
  }
}

impl fmt::Display for Value {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      Value::Unit => write!(f, "()"),
      Value::Function(_, params, _) => write!(f, "Function ({:#?})", params),
      Value::Int64(v) => write!(f, "{}", v),
      Value::Str(v) => write!(f, "{}", v),
      Value::Bool(v) => write!(f, "{}", v),
      Value::BuiltInFunction(_) => write!(f, "BuiltInFunction"),
    }
  }
}

/**
 * Scope forms a spaghetti stack all the way up to the global scope. This is
 * purely a runtime scope (no checks are performed).
 */
#[derive(Debug)]
pub struct Scope {
  // Set to none when the parent of this scope is the global scope.
  pub parent: Option<Rc<RefCell<Scope>>>,
  // The locals of this scope.
  pub locals: HashMap<Identifier, Value>,
}

pub fn push_scope(parent: &Rc<RefCell<Scope>>) -> Rc<RefCell<Scope>> {
  Rc::new(RefCell::new(Scope {
    parent: Some(Rc::clone(parent)),
    locals: HashMap::new(),
  }))
}

impl Scope {
  /**
   * Binds a variable to this scope only.
   */
  pub fn bind_variable(&mut self, identifier: Identifier, value: Value) {
    if self.locals.contains_key(&identifier) {
      panic!(
        "Re-declaration of Ident [{}] at offset [{}-{}] for scope {:#?}",
        identifier.name, identifier.source_ref.left, identifier.source_ref.right, self
      );
    }
    self.locals.insert(identifier, value);
  }

  /**
   * Assigns an already bound variable anywhere up the parent chain.
   */
  pub fn assign_variable(&mut self, identifier: &Identifier, value: Value) {
    if let Some(v) = self.locals.get_mut(identifier) {
      *v = value;
    } else {
      // Check parent (chain)
      if let Some(ref parent_rc) = self.parent {
        return (**parent_rc)
          .borrow_mut()
          .assign_variable(identifier, value);
      }
      // We made it all the way up to globals and it wasn't bound.
      panic!(
        "Ident [{}] never declared before it's use at [{}-{}] for scope {:#?}",
        identifier.name, identifier.source_ref.left, identifier.source_ref.right, self
      );
    }
  }

  /**
   * Assigns a variable to this scope if and only if it is not already bound (shadowed).
   */
  pub fn close_variable(&mut self, identifier: &Identifier, value: Value) {
    if !self.locals.contains_key(&identifier) {
      self.locals.insert(identifier.clone(), value);
    }
  }

  /**
   * Gets a bound variable anywhere up the parent chain.
   */
  pub fn get_variable(&self, identifier: &Identifier) -> Value {
    if self.locals.contains_key(&identifier) {
      return self.locals[identifier].clone();
    } else {
      // Check parent (chain)
      if let Some(ref parent_rc) = self.parent {
        return (**parent_rc).borrow().get_variable(identifier);
      }
    }
    // We got all the way up to globals and it didn't have it as well.
    panic!(
      "Failed to get-bind Ident [{}] defined at [{}-{}] for scope {:#?}",
      identifier.name, identifier.source_ref.left, identifier.source_ref.right, self
    );
  }
}
