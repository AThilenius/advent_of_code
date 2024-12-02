#!/usr/bin/env nu

# The engineers are surprised by the low number of safe reports until they
# realize they forgot to tell you about the Problem Dampener.
#
# The Problem Dampener is a reactor-mounted module that lets the reactor safety
# systems tolerate a single bad level in what would otherwise be a safe report.
# It's like the bad level never happened!
#
# Now, the same rules apply as before, except if removing a single level from
# an unsafe report would make it safe, the report instead counts as safe.
#
# More of the above example's reports are now safe:
#
# 7 6 4 2 1: Safe without removing any level.
# 1 2 7 8 9: Unsafe regardless of which level is removed.
# 9 7 6 2 1: Unsafe regardless of which level is removed.
# 1 3 2 4 5: Safe by removing the second level, 3.
# 8 6 4 4 1: Safe by removing the third level, 4.
# 1 3 6 7 9: Safe without removing any level.
# Thanks to the Problem Dampener, 4 reports are actually safe!
#
# Update your analysis by handling situations where the Problem Dampener can
# remove a single level from unsafe reports. How many reports are now safe?
export def main [
  # Path to the puzzle input
  path: path = "../puzzle-inputs/02-red-nosed-reports.txt",
] {
  open $path
    | lines
    | each { split row " " | into int }
    | each { |set|
      # For each index in the set (array)
      $set | enumerate | each { |e|
        # Create a set without that index, and test that
        $set 
          | without $e.index
          | window 2
          | each { $in.1 - $in.0 }
      }
      # Only one of those sets needs to be true to be considered a pass
      | any { |deltas|
          ($deltas | all { $in in 1..3 }) or ($deltas | all { $in in -1..-3 })
        }
      }
    # Then count up the true ones
    | filter { $in }
    | length
}

# Returns the input array without the given index
def without [
  # The index of the element in the array that should be excluded
  index: number
] list -> list {
  let from = $index
  let to = ($in | length) - $index - 1
  [...($in | take $from), ...($in | last $to)]
}
