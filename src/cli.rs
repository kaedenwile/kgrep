use std::{env, process::exit};

use crate::regex::RegExp;

// This method implements the grep-like cli interface
pub fn cli() {
    let args = env::args();

    let parsed_args = match parse_args(args) {
        Ok(parsed_args) => parsed_args,
        Err(message) => {
            println!("Error: {}", message);
            usage(1);
        }
    };

    if parsed_args.show_help {
        usage(0);
    }

    let Ok(regex) = RegExp::parse(parsed_args.regex.as_str()) else {
        println!("Error: could not parse regular expression.");
        exit(1);
    };

    // TODO: actually read search_path
    let _ = regex.execute(parsed_args.search_path.as_str());
}

#[derive(PartialEq, Debug)]
struct ParsedArgs {
    // Positional args
    regex: String,
    search_path: String,
    // Flags
    show_help: bool,
    case_insensitive: bool,
}

// Expected to be called with raw env::args()
// Will discard first element (program name)
//
// If parsing fails, returns error message to display to user.
fn parse_args(args: impl IntoIterator<Item = String>) -> Result<ParsedArgs, String> {
    let mut regex = None;
    let mut search_path = None;
    let mut show_help = false;
    let mut case_insensitive = false;

    for arg in args.into_iter().skip(1) {
        match arg.as_str() {
            "-h" | "--help" => {
                show_help = true;
            }
            "-i" => {
                case_insensitive = true;
            }
            _ if regex.is_none() => {
                regex = Some(arg);
            }
            _ if search_path.is_none() => {
                search_path = Some(arg);
            }
            _ => return Err(format!("Unknown argument: {}", arg)),
        }
    }

    let Some(regex) = regex else {
        return Err("Missing regular expression".to_string());
    };

    let Some(search_path) = search_path else {
        return Err("Missing search path".to_string());
    };

    return Ok(ParsedArgs {
        regex,
        search_path,
        show_help,
        case_insensitive,
    });
}

pub fn usage(exit_code: i32) -> ! {
    println!("Usage: kgrep regex file [flags]");
    println!("Options: ");
    println!("  --help -h   show this help message and exit");
    println!("  -i          perform case-insensitive match [default=false]");
    exit(exit_code);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_args(command: &str) -> Vec<String> {
        return command.split(" ").map(str::to_string).collect();
    }

    #[test]
    fn parse_args_with_flags() {
        assert_eq!(
            parse_args(make_args("kgrep regex ./path -h -i")),
            Ok(ParsedArgs {
                regex: "regex".to_string(),
                search_path: "./path".to_string(),
                show_help: true,
                case_insensitive: true
            })
        )
    }

    #[test]
    fn parse_args_without_flags() {
        assert_eq!(
            parse_args(make_args("kgrep regex ./path")),
            Ok(ParsedArgs {
                regex: "regex".to_string(),
                search_path: "./path".to_string(),
                show_help: false,
                case_insensitive: false
            })
        )
    }
}
