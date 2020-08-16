// UI utility functions

use std::env::current_exe;
use std::path::Path;
use std::fs;

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

/// Finds the font for the user interface
pub fn find_font () -> Result<String, String> {
  use std::io::Write;
  let mut file = std::fs::File::create("/Users/hendrik/Desktop/log.txt").unwrap();
  // file.write_all(b"Could not find the font path!").unwrap();
  // This function basically only finds the necessary font.
  let __filename = current_exe().unwrap();
  let __dirname = __filename.parent().unwrap();
  let possible_locations = [
    "assets/fonts/lato/Lato-Regular.ttf", // From cargo run, when current_dir points to the crate root
    "../Resources/assets/fonts/lato/Lato-Regular.ttf" // From within a macOS bundle
  ];

  let mut ret: Result<String, String> = Err(String::from("Could not find font file"));

  for path in possible_locations.iter() {
    let font_path = String::from(Path::new(&__dirname).join(path).to_str().unwrap());
    file.write_all(format!("\nLooking at path: {} (Dirname: {:? })", font_path, &__dirname).as_bytes()).unwrap();
    let metadata = fs::metadata(&font_path);
    if let Ok(m) = metadata {
      if m.is_file() {
        ret = Ok(font_path); // Found it!
        break;
      }
    }
  }

  ret // Finally return
}
