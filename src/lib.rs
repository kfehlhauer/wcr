use clap::{Arg, ArgAction, Command};
use std::error::Error;
use std::fs::File;
use std::io::{self, BufRead, BufReader};

type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug)]
pub struct Config {
    files: Vec<String>,
    lines: bool,
    words: bool,
    bytes: bool,
    chars: bool,
}

#[derive(Debug, PartialEq)]
pub struct FileInfo {
    num_lines: usize,
    num_words: usize,
    num_bytes: usize,
    num_chars: usize,
}

pub fn get_args() -> MyResult<Config> {
    let matches = Command::new("wcr")
        .version("0.1.0")
        .author("Kurt Fehlhauer")
        .about("Rust wc")
        .arg(
            Arg::new("files")
                .value_name("FILES")
                .help("Input file(s)")
                .num_args(1..)
                .default_value("-"),
        )
        .arg(
            Arg::new("lines")
                .value_name("LINES")
                .help("Show line count")
                .short('l')
                .long("lines")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("words")
                .value_name("WORDS")
                .help("Show word count")
                .short('w')
                .long("words")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("bytes")
                .value_name("BYTES")
                .help("Show byte count")
                .short('c')
                .long("bytes")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("chars")
                .value_name("CHARS")
                .help("Show character count")
                .short('m')
                .long("chars")
                .action(ArgAction::SetTrue)
                .conflicts_with("bytes"),
        )
        .get_matches();

    let mut lines = matches.get_flag("lines");
    let mut words = matches.get_flag("words");
    let mut bytes = matches.get_flag("bytes");
    let chars = matches.get_flag("chars");

    if !lines && !words && !bytes && !chars {
        lines = true;
        words = true;
        bytes = true;
    }

    Ok(Config {
        files: matches
            .get_many("files")
            .unwrap_or_default()
            .cloned()
            .collect(),
        lines: lines,
        words: words,
        bytes: bytes,
        chars: chars,
    })
}

pub fn run(config: Config) -> MyResult<()> {
    let mut total_lines: usize = 0;
    let mut total_words: usize = 0;
    let mut total_bytes: usize = 0;
    let mut total_chars: usize = 0;

    let mut file_count = 0;
    for filename in &config.files {
        file_count += 1;
        match open(filename) {
            Err(err) => eprintln!("{}: {}", filename, err),
            Ok(f) => match count(f) {
                Ok(file_info) => {
                    total_lines = total_lines + file_info.num_lines;
                    total_words = total_words + file_info.num_words;
                    total_bytes = total_bytes + file_info.num_bytes;
                    total_chars = total_chars + file_info.num_chars;
                    println!(
                        "{}{}{}{}{}",
                        format_field(file_info.num_lines, config.lines),
                        format_field(file_info.num_words, config.words),
                        format_field(file_info.num_bytes, config.bytes),
                        format_field(file_info.num_chars, config.chars),
                        if filename == "-" {
                            "".to_string()
                        } else {
                            format!(" {}", filename)
                        }
                    );
                }
                Err(e2) => eprintln!("{}: {}", filename, e2),
            },
        }
    }
    if file_count > 1 {
        println!(
            "{}{}{}{} total",
            format_field(total_lines, config.lines),
            format_field(total_words, config.words),
            format_field(total_bytes, config.bytes),
            format_field(total_chars, config.chars)
        );
    }
    Ok(())
}

fn format_field(value: usize, show: bool) -> String {
    if show {
        format!("{value:>8}")
    } else {
        "".to_string()
    }
}

fn open(filename: &str) -> MyResult<Box<dyn BufRead>> {
    match filename {
        "-" => Ok(Box::new(BufReader::new(io::stdin()))),
        _ => Ok(Box::new(BufReader::new(File::open(filename)?))),
    }
}

pub fn count(mut file: impl BufRead) -> MyResult<FileInfo> {
    let mut num_lines = 0;

    let mut buffer = String::new();
    while file.read_line(&mut buffer)? > 0 {
        num_lines += 1;
    }

    let num_words = buffer.split_whitespace().count();
    let num_bytes = buffer.as_bytes().len();
    let num_chars = buffer.chars().count();

    Ok(FileInfo {
        num_lines,
        num_words,
        num_bytes,
        num_chars,
    })
}

#[cfg(test)]
mod tests {
    use super::{count, FileInfo};
    use std::io::Cursor;

    #[test]
    fn test_count() {
        let text = "I don't want the world. I just want your half.\r\n";
        let info = count(Cursor::new(text));
        assert!(info.is_ok());
        let expected = FileInfo {
            num_lines: 1,
            num_words: 10,
            num_chars: 48,
            num_bytes: 48,
        };
        assert_eq!(info.unwrap(), expected);
    }

    #[test]
    fn test_count_2() {
        let text = "I don't want the world. I just want your half.\r\n\
        I don't want the world. I just want your half.\r\n";
        let info = count(Cursor::new(text));
        assert!(info.is_ok());
        let expected = FileInfo {
            num_lines: 2,
            num_words: 20,
            num_chars: 96,
            num_bytes: 96,
        };
        assert_eq!(info.unwrap(), expected);
    }
}
