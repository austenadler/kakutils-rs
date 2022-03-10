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
mod kak;
mod math_eval;
mod shuf;
// mod sort;
mod uniq;
use clap::{Parser, Subcommand};
use errors::KakMessage;
pub use kak::*;
use std::env;

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
    // Sort(sort::Options),
    Shuf(shuf::Options),
    Uniq(uniq::Options),
    #[clap(visible_aliases = &["bc", "eval"])]
    MathEval(math_eval::Options),
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

fn run() -> Result<KakMessage, KakMessage> {
    let options = Cli::try_parse().map_err(|e| {
        KakMessage(
            "Error parsing arguments".to_string(),
            Some(format!("Could not parse: {}", e)),
        )
    })?;

    match &options.command {
        // Commands::Sort(o) => sort::sort(o),
        Commands::Shuf(o) => shuf::shuf(o),
        Commands::Uniq(o) => uniq::uniq(o),
        Commands::MathEval(o) => math_eval::math_eval(o),
    }
}

/// # Errors
///
/// Will return `Err` if requested environment variable is not unicode or not present
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
