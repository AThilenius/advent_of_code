extern crate colored;

use colored::*;

use regex::Regex;
use std::collections::HashMap;
use std::ops::Bound::*;

// #[derive(Debug, Clone)]
// pub struct TokenError {
//   pub start: usize,
//   pub message: String,
// }

// impl TokenError {
//   pub fn new(start: usize, message: String) -> TokenError {
//     println!("Token error at: {}\t{}", start, message);
//     TokenError {
//       start: start,
//       message: message,
//     }
//   }
// }

pub struct MetaData {
  pub source_md5: md5::Digest,
  pub longest_offset: usize,
  pub error_at_longest: String,
}

pub type ProductionFn<T> = fn(&str, meta: &mut MetaData, &mut usize) -> Option<T>;

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
  meta: &mut MetaData,
  offset: &mut usize,
) -> Option<&'a str> {
  consume_whitespace(source, offset);
  let stream = &source[*offset as usize..];
  match re.find(stream) {
    Some(mat) => {
      let one = &stream[mat.start()..mat.end()];
      *offset = *offset + mat.end();
      Some(one)
    }
    // Err(TokenError::new(*offset, format!("Expected {} here.", re))),
    None => {
      if *offset > meta.longest_offset {
        meta.longest_offset = *offset;
        meta.error_at_longest = format!("Failed to match regex: {}", re);
      }
      None
    }
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
  meta: &mut MetaData,
  offset: &mut usize,
) -> Option<Vec<&'a str>> {
  let (min, max) = range_to_allowed_match_count(lower_bound, upper_bound);
  let mut matches = vec![];
  for i in 0.. {
    let res = match_regex(re, source, meta, offset);
    match res {
      Some(mat) => {
        matches.push(mat);
        // If we hit the upper bound return out, otherwise carry on matching
        if max >= 0 && i + 1 >= max {
          return Some(matches);
        }
      }
      None => {
        // If we met at least out minimum, return Ok. Otherwise it failed to match
        if i >= min {
          return Some(matches);
        }
        return None;
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
  meta: &mut MetaData,
  offset: &mut usize,
) -> Option<Vec<T>> {
  let (min, max) = range_to_allowed_match_count(lower_bound, upper_bound);
  let mut matches = vec![];
  for i in 0.. {
    let res = production_fn(source, meta, offset);
    match res {
      Some(mat) => {
        matches.push(mat);
        // If we hit the upper bound return out, otherwise carry on matching
        if max >= 0 && i + 1 >= max {
          return Some(matches);
        }
      }
      None => {
        // If we met at least out minimum, return Ok. Otherwise it failed to match
        if i >= min {
          return Some(matches);
        }
        return None;
      }
    }
  }
  unreachable!();
}

// Builtin productions
pub fn end_of_input(source: &str, _meta: &mut MetaData, offset: &mut usize) -> Option<bool> {
  consume_whitespace(source, offset);
  if *offset == source.len() {
    Some(true)
  } else {
    None
  }
}

pub struct Parser {}

impl Parser {
  pub fn new() -> Parser {
    Parser {}
  }

  pub fn parse_or_log_errors<T>(&mut self, production: ProductionFn<T>, source: &str) -> Option<T> {
    let mut offset = 0;
    let mut meta = MetaData {
      source_md5: md5::compute(source),
      longest_offset: 0,
      error_at_longest: "Failed to start parser".to_owned(),
    };
    let res = production(source, &mut meta, &mut offset);
    match res {
      Some(prod) => Some(prod),
      None => {
        // Get the entire line where the error happened
        let mut line_offsets = vec![];
        let mut i = 0;
        for line in source.lines() {
          // Need to add 1 for the newline character that got trimmed
          line_offsets.push((line, i, i + line.len() + 1));
          i += line.len() + 1;
        }
        // Find the line
        let mut on_line = 0;
        let target_line_search = line_offsets.iter().find(|(_, s, e)| {
          on_line += 1;
          *s <= meta.longest_offset && *e >= meta.longest_offset
        });
        if let Some((line, s, _)) = target_line_search {
          println!();
          println!(
            "Failed to parse line {}, column {}:",
            on_line,
            meta.longest_offset - *s
          );
          println!("{}", line.blue());
          let offset_in_line = meta.longest_offset - *s;
          for _ in 0..offset_in_line {
            print!(" ");
          }
          println!("^ {}", meta.error_at_longest.red());
          println!();
        } else {
          println!(
            "Failed at {}/{} with: {}",
            meta.longest_offset,
            source.len(),
            meta.error_at_longest.red()
          );
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
  ([$name:ident], $src:ident, $meta:ident, $offset:ident) => {{
    $name($src, $meta, &mut $offset)?
  }};
  ([$regex:expr], $src:ident, $meta:ident, $offset:ident) => {{
    lazy_static! {
      static ref RE: regex::Regex = regex::Regex::new(&format!(r"^{}", $regex)).unwrap();
    }
    crate::parser::match_regex(&RE, $src, $meta, &mut $offset)?
  }};
  ([$num:expr; $name:ident], $src:ident, $meta:ident, $offset:ident) => {{
    crate::parser::match_ident_range(
      std::ops::RangeBounds::start_bound(&$num),
      std::ops::RangeBounds::end_bound(&$num),
      $name,
      $src,
      $meta,
      &mut $offset,
    )?
  }};
  ([$num:expr; $regex:expr], $src:ident, $meta:ident, $offset:ident) => {{
    lazy_static! {
      static ref RE: regex::Regex = regex::Regex::new(&format!(r"^\s*{}", $regex)).unwrap();
    }
    crate::parser::match_regex_range(
      std::ops::RangeBounds::start_bound(&$num),
      std::ops::RangeBounds::end_bound(&$num),
      &RE,
      $src,
      $meta,
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
        #[allow(dead_code)]
        pub fn $name(
            source: &str,
            meta: &mut crate::parser::MetaData,
            offset: &mut usize
        ) -> Option<$ret_type> {
          use std::collections::HashMap;
          use std::sync::Mutex;
          lazy_static! {
            static ref CACHE: Mutex<HashMap<
              (md5::Digest, usize), Option<(usize, $ret_type)>>>
              = Mutex::new(HashMap::new());
          }
          // Return memoized cache if we have one (also need to increment offset).
          if let Some(ref cache) = CACHE.lock().unwrap().get(&(meta.source_md5, *offset)) {
            match cache.clone() {
              Some((new_offset, v)) => {
                *offset = *new_offset;
                 return Some(v.to_owned());
              }
              None => return None
            }
          }
          #[allow(unused_assignments)]
          let mut arm_results: Option<$ret_type> = None;
          $(
            arm_results = || -> Option<$ret_type> {
              // Offset is not advanced unless the entire arm matches.
              let mut local_offset = *offset;
              // A single production arm
              $(
                let $mat_name = production_match_expressions!{$decl, source, meta, local_offset};
              )*
              // The entire arm matched, we can advance offset.
              let final_expression_value = $ret_expr;
              CACHE.lock().unwrap().insert((meta.source_md5, *offset),
                  Some((local_offset, final_expression_value.clone())));
              *offset = local_offset;
              return Some(final_expression_value);
            }();
            if let Some(res) = arm_results {
              return Some(res);
            }
          )*
          CACHE.lock().unwrap().insert((meta.source_md5, *offset), None);
          None
        }

    )*
  };
}
