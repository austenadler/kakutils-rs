#![warn(clippy::all, clippy::pedantic, clippy::nursery, clippy::cargo)]

mod errors;
use alphanumeric_sort::compare_str;
use clap::Parser;
use clap::Subcommand;
use errors::KakMessage;
use regex::Regex;
use std::env;
use std::fs;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::Write;

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
    // #[clap(flatten)]
    Sort(SortOptions),
}

#[derive(clap::StructOpt, Debug)]
struct SortOptions {
    #[clap(index = 1)]
    regex: Option<String>,
    #[clap(short = 's', long)]
    subselections_register: Option<char>,
    // TODO: Can we invert a boolean? This name is terrible
    #[clap(short = 'S', long)]
    no_skip_whitespace: bool,
    #[clap(short, long)]
    lexicographic_sort: bool,
    #[clap(short, long)]
    reverse: bool,
    #[clap(short, long)]
    ignore_case: bool,
}

struct SortableSelection {
    content: &str,
    subselections: Vec<&str>,
}

fn main() {
    let msg = match run() {
        Ok(msg) => msg,
        Err(msg) => {
            eprintln!("{} (Debug info: {:?})", msg.0, msg.1);
            msg
        }
    };

    if let Err(e) = send_message(&msg) {
        println!("{:?}", e);
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
            Some(format!("Could not parse: {:?}", e)),
        )
    })?;

    match &options.command {
        Commands::Sort(sort_options) => sort(sort_options),
    }
}

fn sort_keys_simple<'a>(
    sort_options: &SortOptions,
    selections: &[&str],
) -> Result<Vec<SortableSelection>, KakMessage> {
    let re = sort_options
        .regex
        .as_ref()
        .map(|r| Regex::new(r))
        .transpose()
        .map_err(|_| {
            format!(
                "Invalid regular expression: {}",
                sort_options.regex.as_ref().unwrap_or(&"".to_string())
            )
        })?;

    Ok(selections
        .iter()
        .map(|s| {
            if sort_options.no_skip_whitespace {
            SortableSelection{
                content: s,
                subselections: vec![s],
                }
            } else {
            SortableSelection{
                content: s,
                subselections: vec![s.trim()],
                }
            }
        })
        .map(|(s, k)| {
            let captures = re.as_ref()?.captures(k)?;
            captures
                .get(1)
                .or_else(|| captures.get(0))
                .map(|m| m.as_str())
        })
        .collect::<Vec<Option<&str>>>())
}

// fn sort_keys_subselection(sort_options: &SortOptions) -> Result<Vec<(&String, Option<&str>)>, KakMessage> {
//     let sort_selections = kak_response("%val{selections}")?.iter_mut().map(|a| {
//             if sort_options.no_skip_whitespace {
//                 a
//             } else {
//                 a.trim()
//             }
//         });
//     let sort_selections_desc = kak_response("%val{selections_desc}")?;
//     kak_exec("z")?;
//     let selections = kak_response("%val{selections}")?;
//     let selections_desc = kak_response("%val{selections_desc}")?;
// }

fn sort(sort_options: &SortOptions) -> Result<KakMessage, KakMessage> {
    // let selections = kak_response("%val{selections}")?;

    // let sort_keys = if let Some(r) = sort_options.subselections_register {
    //     let selections_desc = kak_response("%val{selections_desc}")?;
    // } else {
    // };

    let mut zipped = match sort_options.subselections_register {
        Some(c) => {
            let selections = kak_response("%val{selections}")?;
            selections
                .into_iter()
                .zip(sort_keys_simple(sort_options, &selections))
        }
        None => {
            let selections = kak_response("%val{selections}")?;
            selections.iter().zip(selections.iter().map(|s| s.as_str()))
        }
    };

    // let mut zipped = sort_keys_simple(sort_options)?;

    zipped.sort_by(|(a, a_key), (b, b_key)| {
        let a = a_key.unwrap_or(a);
        let b = b_key.unwrap_or(b);

        if sort_options.lexicographic_sort {
            a.cmp(b)
        } else {
            compare_str(a, b)
        }
    });

    let mut f = open_command_fifo()?;

    write!(f, "reg '\"'")?;

    let iter: Box<dyn Iterator<Item = _>> = if sort_options.reverse {
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
        format!("Sorted {} selections", zipped.len()),
        None,
    ))
}

fn kak_exec(cmd: &str) -> Result<(), KakMessage> {
    let mut f = open_command_fifo()?;

    write!(f, "{}", cmd).map_err(|e| e.into())
}

fn kak_response(msg: &str) -> Result<Vec<String>, KakMessage> {
    kak_exec(&format!(
        "echo -quoting shell -to-file {} -- {msg}",
        get_var("kak_response_fifo")?
    ))?;

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
