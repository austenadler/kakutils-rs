#![warn(clippy::all, clippy::pedantic, clippy::nursery, clippy::cargo)]

mod errors;
use alphanumeric_sort::compare_str;
use clap::Parser;
use errors::KakMessage;
use regex::Regex;
use std::env;
use std::fs;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::Write;

#[derive(Parser, Debug)]
#[clap(about, version, author)]
struct Options {
    // TODO: Allow clap to parse these. Currently clap treats them as positional
    // #[clap(env = "kak_command_fifo", takes_value = false)]
    // kak_command_fifo_name: PathBuf,
    // #[clap(env = "kak_response_fifo", takes_value = false)]
    // kak_response_fifo_name: PathBuf,
    #[clap(index = 1)]
    regex: Option<String>,
    #[clap(short = 'S', long)]
    // TODO: Can we invert a boolean? This name is terrible
    no_skip_whitespace: bool,
    #[clap(short, long)]
    lexicographic_sort: bool,
    #[clap(short, long)]
    reverse: bool,
}

fn main() {
    let msg = match run() {
        Ok(msg) => msg,
        Err(msg) => {
            eprintln!("{} (Debug info: {:?})", msg.0, msg.1);
            msg
        }
    };

    send_message(&msg);
}

fn send_message(msg: &KakMessage) {
    let msg_str = msg.0.replace('\'', "''");
    {
        let mut f = open_command_fifo().unwrap();

        write!(f, "echo '{}';", msg_str).unwrap();
        write!(f, "echo -debug '{}';", msg_str).unwrap();

        if let Some(debug_msg_str) = &msg.1 {
            write!(f, "echo -debug '{}';", debug_msg_str.replace('\'', "''")).unwrap();
        }
    }
}

fn run() -> Result<KakMessage, KakMessage> {
    let options = Options::try_parse().map_err(|e| {
        KakMessage(
            "Error parsing arguments".to_string(),
            Some(format!("Could not parse: {:?}", e)),
        )
    })?;

    let re = options
        .regex
        .as_ref()
        .map(|r| Regex::new(r))
        .transpose()
        .map_err(|_| {
            format!(
                "Invalid regular expression: {}",
                options.regex.unwrap_or("".to_string())
            )
        })?;

    let selections = read_selections()?;

    let mut zipped = selections
        .iter()
        .zip(
            selections
                .iter()
                .map(|a| {
                    if options.no_skip_whitespace {
                        a
                    } else {
                        a.trim()
                    }
                })
                .map(|a| {
                    let captures = re.as_ref()?.captures(a)?;
                    captures
                        .get(1)
                        .or_else(|| captures.get(0))
                        .map(|m| m.as_str())
                }),
        )
        .collect::<Vec<(&String, Option<&str>)>>();

    zipped.sort_by(|(a, a_key), (b, b_key)| {
        let a = a_key.unwrap_or(a);
        let b = b_key.unwrap_or(b);

        if options.lexicographic_sort {
            a.cmp(b)
        } else {
            compare_str(a, b)
        }
    });

    let mut f = open_command_fifo()?;

    write!(f, "reg '\"'")?;

    let iter: Box<dyn Iterator<Item = _>> = if options.reverse {
        Box::new(zipped.iter().rev())
    } else {
        Box::new(zipped.iter())
    };

    for i in iter {
        let new_selection = i.0.replace('\'', "''");
        write!(f, " '{}'", new_selection)?;
    }
    write!(f, " ; exec R;")?;

    Ok(KakMessage(
        format!("Sorted {} selections", selections.len()),
        None,
    ))
}

fn read_selections() -> Result<Vec<String>, KakMessage> {
    {
        let mut f = open_command_fifo()?;

        write!(
            f,
            "echo -quoting shell -to-file {} -- %val{{selections}}",
            get_var("kak_response_fifo")?
        )?;
    }

    let selections = shellwords::split(&fs::read_to_string(&get_var("kak_response_fifo")?)?)?;

    Ok(selections)
}

fn open_command_fifo() -> Result<File, KakMessage> {
    OpenOptions::new()
        .write(true)
        .append(true)
        .open(&get_var("kak_command_fifo")?)
        .map_err(|e| e.into())
}

fn get_var(var_name: &str) -> Result<String, KakMessage> {
    env::var(var_name).map_err(|e| match e {
        env::VarError::NotPresent => {
            KakMessage(format!("Env var {} is not defined", var_name), None)
        }
        env::VarError::NotUnicode(_) => {
            KakMessage(format!("Env var {} is not unicode", var_name), None)
        }
    })
}
