extern crate colored;

use colored::*;

use regex::Regex;
use std::collections::HashMap;
use std::ops::Bound::*;

#[derive(Debug)]
pub struct TokenError {
  pub start: usize,
  pub message: String,
}

pub type ProductionFn<T> = fn(
  &str,
  &mut std::collections::HashMap<&'static str, regex::Regex>,
  &mut usize,
) -> Result<T, crate::parser::TokenError>;

pub fn match_regex<'a>(
  re_text: &'static str,
  source: &'a str,
  regex_atlas: &mut HashMap<&'static str, Regex>,
  offset: &mut usize,
) -> Result<&'a str, TokenError> {
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
  let re = regex_atlas
    .entry(re_text)
    .or_insert_with(|| Regex::new(&format!(r"^\s*{}", re_text)).unwrap());
  match re.find(stream) {
    Some(mat) => {
      let one = &stream[mat.start()..mat.end()];
      *offset = *offset + mat.end();
      Ok(one)
    }
    None => Err(TokenError {
      start: *offset,
      message: format!("Expected {} here.", re_text),
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
  re_text: &'static str,
  source: &'a str,
  regex_atlas: &mut HashMap<&'static str, Regex>,
  offset: &mut usize,
) -> Result<Vec<&'a str>, TokenError> {
  let (min, max) = range_to_allowed_match_count(lower_bound, upper_bound);
  let mut matches = vec![];
  for i in 0.. {
    let res = match_regex(re_text, source, regex_atlas, offset);
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
pub fn match_ident_range<'a, T>(
  lower_bound: std::collections::Bound<&i32>,
  upper_bound: std::collections::Bound<&i32>,
  production_fn: ProductionFn<T>,
  source: &'a str,
  regex_atlas: &mut HashMap<&'static str, Regex>,
  offset: &mut usize,
) -> Result<Vec<T>, TokenError> {
  let (min, max) = range_to_allowed_match_count(lower_bound, upper_bound);
  let mut matches = vec![];
  for i in 0.. {
    let res = production_fn(source, regex_atlas, offset);
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

/**
 * Matches a single production arm expression, like name:[0..1; r"foobar"] or _:[ident].
 */
macro_rules! production_match_expressions {
  ([$name:ident], $src:ident, $regex_atlas:ident, $offset:ident) => {{
    $name($src, $regex_atlas, &mut $offset)?
  }};
  ([$regex:expr], $src:ident, $regex_atlas:ident, $offset:ident) => {{
    crate::parser::match_regex(&$regex, $src, $regex_atlas, &mut $offset)?
  }};
  ([$num:expr; $name:ident], $src:ident, $regex_atlas:ident, $offset:ident) => {{
    crate::parser::match_ident_range(
      std::ops::RangeBounds::start_bound(&$num),
      std::ops::RangeBounds::end_bound(&$num),
      $name,
      $src,
      $regex_atlas,
      &mut $offset,
    )?
  }};
  ([$num:expr; $regex:expr], $src:ident, $regex_atlas:ident, $offset:ident) => {{
    crate::parser::match_regex_range(
      std::ops::RangeBounds::start_bound(&$num),
      std::ops::RangeBounds::end_bound(&$num),
      &$regex,
      $src,
      $regex_atlas,
      &mut $offset,
    )?
  }};
}

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
      fn $name<'a>(
          source: &'a str,
          regex_atlas: &mut std::collections::HashMap<&'static str, regex::Regex>,
          offset: &mut usize)
        -> Result<$ret_type, crate::parser::TokenError> {
        #[allow(unused_assignments)]
        let mut arm_results: Result<$ret_type, crate::parser::TokenError> = Err(
          crate::parser::TokenError{start: *offset, message: "No match arms specified in production.".to_owned()}
        );
        $(
          arm_results = || -> Result<$ret_type, crate::parser::TokenError> {
            // Offset is not advanced unless the entire arm matches.
            let mut local_offset = *offset;
            // A single production arm
            $(
              let $mat_name = production_match_expressions!{
                $decl, source, regex_atlas, local_offset};
            )*
            // The entire arm matched, we can advance offset.
            *offset = local_offset;
            return Ok($ret_expr);
          }();
          if let Ok(res) = arm_results {
            return Ok(res);
          }
        )*
        arm_results
      }
    )*
  };
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

  pub fn parse_or_log_errors<'a, T>(
    &mut self,
    production: ProductionFn<T>,
    source: &'a str,
  ) -> Option<T> {
    let mut offset = 0;
    let res = production(source, &mut self.regex_atlas, &mut offset);
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
