// UI utility functions

pub fn cursor_in_rect (point: [f64; 2], rect: [f64; 4]) -> bool {
  point[0] > rect[0] && point[0] < rect[0] + rect[2] && point[1] > rect[1] && point[1] < rect[1] + rect[3]
}

pub fn format_number (number: f64) -> String {
  // Number: 1_465.435
  let mut as_chars: Vec<char> = Vec::new();

  let remainder = number - number.floor();

  // First convert the number and split it up by digits (use i32 type casting to prevent .0 as appendix)
  for digit in ((number - remainder) as i32).to_string().chars() {
    as_chars.push(digit);
  }

  // Number: 1, 4, 5, 6

  // Now reverse, and recreate the vector using signs
  as_chars.reverse();

  // Number: 6, 5, 4, 1

  let mut chars_with_sep: Vec<char> = Vec::new();

  for (i, ch) in as_chars.into_iter().enumerate() {
    if i > 0 && i % 3 == 0 {
      chars_with_sep.push(',');
    }

    chars_with_sep.push(ch);
  }

  // Final reversal
  chars_with_sep.reverse();

  // Finally, build the return string
  let mut ret = String::new();

  for ch in chars_with_sep {
    ret.push(ch);
  }

  if remainder > 0.0 {
    format!("{}.{}", ret, remainder.to_string().split_off(2))
  } else {
    ret // Without remainder
  }
}
