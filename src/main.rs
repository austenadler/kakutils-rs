// Enable clippy 'hard mode'
#![warn(clippy::all, clippy::pedantic, clippy::nursery)]
// Intended behavior (10_f64 as i32)
#![allow(clippy::cast_possible_truncation)]
// Cannot be fixed
#![allow(clippy::multiple_crate_versions)]
#![allow(clippy::struct_excessive_bools)]

// #![warn(clippy::all, clippy::pedantic, clippy::nursery, clippy::cargo)]
// #![allow(dead_code, unused_imports)]

mod errors;
mod math_eval;
mod shuf;
mod sort;
mod uniq;
use clap::{Parser, Subcommand};
use errors::KakMessage;
use std::{
    env, fs,
    fs::{File, OpenOptions},
    io::Write,
    str::FromStr,
};

#[derive(Parser, Debug)]
#[clap(about, version, author)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
    // TODO: Allow clap to parse these. Currently clap treats them as positional
    // #[clap(env = "kak_command_fifo", takes_value = false)]
    // kak_command_fifo_name: PathBuf,
    // #[clap(env = "kak_response_fifo", takes_value = false)]
    // kak_response_fifo_name: PathBuf,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Sort(sort::Options),
    Shuf(shuf::Options),
    Uniq(uniq::Options),
    #[clap(visible_aliases = &["bc", "eval"])]
    MathEval(math_eval::Options),
}

#[derive(PartialEq, PartialOrd, Ord, Eq, Debug)]
pub struct SelectionDesc {
    left: AnchorPosition,
    right: AnchorPosition,
}

impl FromStr for SelectionDesc {
    type Err = KakMessage;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (left, right) = s.split_once(',').ok_or_else(|| {
            KakMessage(
                "Could not parse position".to_string(),
                Some(format!("Could not parse as position: {}", s)),
            )
        })?;

        Ok(Self {
            left: AnchorPosition::from_str(left)?,
            right: AnchorPosition::from_str(right)?,
        })
    }
}

#[derive(PartialOrd, PartialEq, Clone, Eq, Ord, Debug)]
pub struct AnchorPosition {
    row: usize,
    col: usize,
}
impl FromStr for AnchorPosition {
    type Err = KakMessage;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (left, right) = s.split_once('.').ok_or_else(|| {
            KakMessage(
                "Could not parse position".to_string(),
                Some(format!("Could not parse as position: {}", s)),
            )
        })?;
        Ok(Self {
            row: usize::from_str(left)?,
            col: usize::from_str(right)?,
        })
    }
}

impl SelectionDesc {
    fn sort(&self) -> Self {
        if self.left < self.right {
            // left anchor is first
            Self {
                left: self.left.clone(),
                right: self.right.clone(),
            }
        } else {
            // right anchor is first
            Self {
                left: self.right.clone(),
                right: self.left.clone(),
            }
        }
    }

    fn contains(&self, b: &Self) -> bool {
        // Cursor and anchor can be flipped. Set a.0 to be leftmost cursor
        let sorted_a = self.sort();
        let sorted_b = b.sort();

        sorted_b.left >= sorted_a.left && sorted_b.right <= sorted_a.right
    }
}

fn main() {
    let msg = match run() {
        Ok(msg) => msg,
        Err(msg) => {
            // TODO: Do not do a string allocation here
            eprintln!(
                "{} (Debug info: {})",
                msg.0,
                msg.1.as_ref().map_or("", String::as_str)
            );
            msg
        }
    };

    if let Err(e) = send_message(&msg) {
        println!("{}", e);
    }
}

fn send_message(msg: &KakMessage) -> Result<(), Box<dyn std::error::Error>> {
    let msg_str = msg.0.replace('\'', "''");
    {
        let mut f =
            open_command_fifo().map_err(|e| format!("Could not open command fifo: {:?}", e))?;

        write!(f, "echo '{}';", msg_str)?;
        write!(f, "echo -debug '{}';", msg_str)?;

        if let Some(debug_msg_str) = &msg.1 {
            write!(f, "echo -debug '{}';", debug_msg_str.replace('\'', "''"))?;
        }
    }
    Ok(())
}

fn run() -> Result<KakMessage, KakMessage> {
    let options = Cli::try_parse().map_err(|e| {
        KakMessage(
            "Error parsing arguments".to_string(),
            Some(format!("Could not parse: {}", e)),
        )
    })?;

    match &options.command {
        Commands::Sort(o) => sort::sort(o),
        Commands::Shuf(o) => shuf::shuf(o),
        Commands::Uniq(o) => uniq::uniq(o),
        Commands::MathEval(o) => math_eval::math_eval(o),
    }
}

pub fn kak_exec(cmd: &str) -> Result<(), KakMessage> {
    let mut f = open_command_fifo()?;

    write!(f, "{}", cmd).map_err(|e| e.into())
}

pub fn kak_response(msg: &str) -> Result<Vec<String>, KakMessage> {
    kak_exec(&format!(
        "echo -quoting shell -to-file {} -- {msg}",
        get_var("kak_response_fifo")?
    ))?;

    let selections = shellwords::split(&fs::read_to_string(&get_var("kak_response_fifo")?)?)?;

    Ok(selections)
}

pub fn open_command_fifo() -> Result<File, KakMessage> {
    OpenOptions::new()
        .write(true)
        .append(true)
        .open(&get_var("kak_command_fifo")?)
        .map_err(|e| e.into())
}

pub fn get_var(var_name: &str) -> Result<String, KakMessage> {
    env::var(var_name).map_err(|e| match e {
        env::VarError::NotPresent => {
            KakMessage(format!("Env var {} is not defined", var_name), None)
        }
        env::VarError::NotUnicode(_) => {
            KakMessage(format!("Env var {} is not unicode", var_name), None)
        }
    })
}

#[cfg(test)]
mod test {
    use super::*;
    const sd: SelectionDesc = SelectionDesc {
        left: AnchorPosition { row: 18, col: 9 },
        right: AnchorPosition { row: 10, col: 1 },
    };
    #[test]
    fn test_anchor_position() {
        // let sd = SelectionDesc {
        //     left: AnchorPosition { row: 18, col: 9 },
        //     right: AnchorPosition { row: 10, col: 0 },
        // };
        // Check parsing
        assert_eq!(SelectionDesc::from_str("18.9,10.1").unwrap(), sd);
        // Check if multiple parsed ones match
        assert_eq!(
            SelectionDesc::from_str("18.9,10.1").unwrap(),
            SelectionDesc::from_str("18.9,10.1").unwrap()
        );
        // Check if sorting works
        assert_eq!(sd.sort(), SelectionDesc::from_str("10.1,18.9").unwrap());
        assert_eq!(sd.sort(), sd.sort().sort());
    }

    #[test]
    fn test_contains() {
        assert_true!(sd.contains(sd));
        assert_false!(sd.contains(SelectionDesc::from_str("17.9,10.1").unwrap()))
        assert_false!(sd.contains(SelectionDesc::from_str("18.8,10.1").unwrap()))
        assert_false!(sd.contains(SelectionDesc::from_str("18.9,11.1").unwrap()))
        assert_false!(sd.contains(SelectionDesc::from_str("18.9,10.2").unwrap()))
        assert_true!(sd.contains(SelectionDesc::from_str("19.9,10.1").unwrap()))
        assert_true!(sd.contains(SelectionDesc::from_str("18.10,10.1").unwrap()))
        assert_true!(sd.contains(SelectionDesc::from_str("18.9,9.1").unwrap()))
        assert_true!(sd.contains(SelectionDesc::from_str("18.9,10.0").unwrap()))
    }
}
