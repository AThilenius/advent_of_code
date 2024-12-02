import gleam/result
import gleam/string
import argv
import gleam/io
import simplifile.{read}

pub fn main() {
  case argv.load().arguments {
    [path] -> parse_file(path)
    _ -> io.println("usage: ./program hello <name>")
  }
}

fn parse_file(path: String) {
  use content <- result.try(read(path))
  let lines = string.split(content, "\n")
  io.println(content)
}

fn to_left_and_right_lists(left: List(String), right: List(String), input: List(String)) {
  case rest {
    [first, ..] -> case string.split(first, "   ") {
      todo
    }
  }
}
