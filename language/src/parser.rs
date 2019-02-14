extern crate colored;

use colored::*;

use regex::Regex;
use std::collections::HashMap;
use std::ops::Bound::*;
use std::sync::Mutex;

#[derive(Debug, Clone)]
pub struct TokenError {
  pub start: usize,
  pub message: String,
}

pub type ProductionFn<T> = fn(&str, &mut usize) -> Result<T, crate::parser::TokenError>;

fn consume_whitespace(source: &str, offset: &mut usize) {
  let mut stream = &source[*offset as usize..];
  // Skip whitespace and comments
  lazy_static! {
    static ref whitespace_re: Regex = regex::Regex::new(r"^\s+").unwrap();
    static ref comment_re: Regex = regex::Regex::new(r"^\s*//[^\n]*\n").unwrap();
  }
  loop {
    if let Some(m) = whitespace_re.find(stream) {
      *offset += m.end();
      stream = &source[*offset as usize..];
    } else if let Some(m) = comment_re.find(stream) {
      *offset += m.end();
      stream = &source[*offset as usize..];
    } else {
      break;
    }
  }
}

pub fn match_regex<'a>(
  re: &Regex,
  source: &'a str,
  offset: &mut usize,
) -> Result<&'a str, TokenError> {
  consume_whitespace(source, offset);
  let stream = &source[*offset as usize..];
  match re.find(stream) {
    Some(mat) => {
      let one = &stream[mat.start()..mat.end()];
      *offset = *offset + mat.end();
      Ok(one)
    }
    None => Err(TokenError {
      start: *offset,
      message: format!("Expected {} here.", re),
    }),
  }
}

#[allow(dead_code)]
fn range_to_allowed_match_count(
  lower_bound: std::collections::Bound<&i32>,
  upper_bound: std::collections::Bound<&i32>,
) -> (i32, i32) {
  match (lower_bound, upper_bound) {
    (Unbounded, Unbounded) => (0, -1),
    (Unbounded, Included(e)) => (0, *e),
    (Unbounded, Excluded(e)) => (0, *e),
    (Included(s), Unbounded) => (*s, -1),
    (Excluded(s), Unbounded) => (*s, -1),
    (Included(s), Included(e)) => (*s, *e),
    (Excluded(s), Included(e)) => (*s, *e),
    (Included(s), Excluded(e)) => (*s, *e),
    (Excluded(s), Excluded(e)) => (*s, *e),
  }
}

#[allow(dead_code)]
pub fn match_regex_range<'a>(
  lower_bound: std::collections::Bound<&i32>,
  upper_bound: std::collections::Bound<&i32>,
  re: &Regex,
  source: &'a str,
  offset: &mut usize,
) -> Result<Vec<&'a str>, TokenError> {
  let (min, max) = range_to_allowed_match_count(lower_bound, upper_bound);
  let mut matches = vec![];
  for i in 0.. {
    let res = match_regex(re, source, offset);
    match res {
      Ok(mat) => {
        matches.push(mat);
        // If we hit the upper bound return out, otherwise carry on matching
        if max >= 0 && i + 1 >= max {
          return Ok(matches);
        }
      }
      Err(e) => {
        // If we met at least out minimum, return Ok. Otherwise it failed to match
        if i >= min {
          return Ok(matches);
        }
        return Err(e);
      }
    }
  }
  unreachable!();
}

#[allow(dead_code)]
pub fn match_ident_range<T>(
  lower_bound: std::collections::Bound<&i32>,
  upper_bound: std::collections::Bound<&i32>,
  production_fn: ProductionFn<T>,
  source: &str,
  offset: &mut usize,
) -> Result<Vec<T>, TokenError> {
  let (min, max) = range_to_allowed_match_count(lower_bound, upper_bound);
  let mut matches = vec![];
  for i in 0.. {
    let res = production_fn(source, offset);
    match res {
      Ok(mat) => {
        matches.push(mat);
        // If we hit the upper bound return out, otherwise carry on matching
        if max >= 0 && i + 1 >= max {
          return Ok(matches);
        }
      }
      Err(e) => {
        // If we met at least out minimum, return Ok. Otherwise it failed to match
        if i >= min {
          return Ok(matches);
        }
        return Err(e);
      }
    }
  }
  unreachable!();
}

// Builtin productions
pub fn end_of_input(source: &str, offset: &mut usize) -> Result<bool, crate::parser::TokenError> {
  consume_whitespace(source, offset);
  if *offset == source.len() {
    Ok(true)
  } else {
    Err(TokenError {
      start: *offset,
      message: "expected end of input".to_owned(),
    })
  }
}

pub struct Parser {
  regex_atlas: HashMap<&'static str, Regex>,
}

impl Parser {
  pub fn new() -> Parser {
    Parser {
      regex_atlas: HashMap::new(),
    }
  }

  pub fn parse_or_log_errors<T>(&mut self, production: ProductionFn<T>, source: &str) -> Option<T> {
    let mut offset = 0;
    let res = production(source, &mut offset);
    match res {
      Ok(prod) => Some(prod),
      Err(err) => {
        // Get the entire line where the error happened
        let mut line_offsets = vec![];
        let mut i = 0;
        for line in source.lines() {
          line_offsets.push((line, i, i + line.len()));
          i += line.len();
        }
        // Find the line
        let mut on_line = 0;
        let target_line_search = line_offsets.iter().find(|(_, s, e)| {
          on_line += 1;
          *s <= err.start && *e >= err.start
        });
        if let Some((line, s, _)) = target_line_search {
          println!();
          println!(
            "Failed to parse line {}, column {}:",
            on_line,
            err.start - *s
          );
          println!("{}", line.blue());
          let offset_in_line = err.start - *s;
          for _ in 0..offset_in_line {
            print!(" ");
          }
          println!("^ {}", err.message.red());
          println!();
        }
        None
      }
    }
  }
}

/**
 * Matches a single production arm expression, like name:[0..1; r"foobar"] or _:[ident].
 */
macro_rules! production_match_expressions {
  ([$name:ident], $src:ident, $offset:ident) => {{
    $name($src, &mut $offset)?
  }};
  ([$regex:expr], $src:ident, $offset:ident) => {{
    lazy_static! {
      static ref RE: regex::Regex = regex::Regex::new(&format!(r"^\s*{}", $regex)).unwrap();
    }
    crate::parser::match_regex(&RE, $src, &mut $offset)?
  }};
  ([$num:expr; $name:ident], $src:ident, $offset:ident) => {{
    crate::parser::match_ident_range(
      std::ops::RangeBounds::start_bound(&$num),
      std::ops::RangeBounds::end_bound(&$num),
      $name,
      $src,
      &mut $offset,
    )?
  }};
  ([$num:expr; $regex:expr], $src:ident, $offset:ident) => {{
    lazy_static! {
      static ref RE: regex::Regex = regex::Regex::new(&format!(r"^\s*{}", $regex)).unwrap();
    }
    crate::parser::match_regex_range(
      std::ops::RangeBounds::start_bound(&$num),
      std::ops::RangeBounds::end_bound(&$num),
      &RE,
      $src,
      &mut $offset,
    )?
  }};
}

#[derive(Clone)]
enum TestEnum {}

/**
 * The main productions macro. This enumerates each production and creates a fn for it.
 */
macro_rules! productions {
  (
    $(
      $name:ident -> $ret_type:ty {
        $(
          { $( $mat_name:tt: $decl:tt )* } => $ret_expr:expr
        ),* $(,)*
      }
    )*
  ) => {
    $(
        #[allow(dead_code)]
        pub fn $name(
            source: &str,
            offset: &mut usize
        ) -> Result<$ret_type, crate::parser::TokenError> {
          use std::collections::HashMap;
          use std::sync::Mutex;
          lazy_static! {
            static ref CACHE: Mutex<HashMap<
              usize,
              Result<(usize, $ret_type), crate::parser::TokenError>>>
              = Mutex::new(HashMap::new());
          }
          // Return memoized cache if we have one (also need to increment offset).
          if let Some(ref cache) = CACHE.lock().unwrap().get(&offset) {
            match cache.clone() {
              Ok((new_offset, v)) => {
                *offset = *new_offset;
                 return Ok(v.to_owned());
              }
              Err(e) => return Err(e.to_owned()),
            }
          }
          #[allow(unused_assignments)]
          let no_arm_err = crate::parser::TokenError{
            start: *offset,
            message: "No match arms specified in production.".to_owned()
          };
          let mut arm_results: Result<$ret_type, crate::parser::TokenError> = Err(no_arm_err.clone());
          let mut all_errors = vec![];
          all_errors.push(no_arm_err);
          $(
            arm_results = || -> Result<$ret_type, crate::parser::TokenError> {
              // Offset is not advanced unless the entire arm matches.
              let mut local_offset = *offset;
              // A single production arm
              $(
                let $mat_name = production_match_expressions!{$decl, source, local_offset};
              )*
              // The entire arm matched, we can advance offset.
              let final_expression_value = $ret_expr;
              CACHE.lock().unwrap().insert(*offset, Ok((local_offset, final_expression_value.clone())));
              *offset = local_offset;
              return Ok(final_expression_value);
            }();
            match arm_results {
              Ok(res) => {
                return Ok(res);
              },
              Err(err) => all_errors.push(err),
            }
          )*
          // Return the longest error
          let longest_error = all_errors.iter().max_by_key(|e| e.start).unwrap();
          CACHE.lock().unwrap().insert(*offset, Err(longest_error.clone()));
          Err(longest_error.clone())
        }

    )*
  };
}
