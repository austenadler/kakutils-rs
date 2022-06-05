// Enable clippy 'hard mode'
#![warn(clippy::all, clippy::pedantic, clippy::nursery)]
// Intended behavior (10_f64 as i32)
#![allow(clippy::cast_possible_truncation)]
// Cannot be fixed
#![allow(clippy::multiple_crate_versions)]
#![allow(clippy::struct_excessive_bools)]

mod errors;
mod math_eval;
mod shuf;
mod sort;
mod stdin;
mod trim;
mod uniq;
mod xargs;
use clap::{Parser, Subcommand};
use kakplugin::{display_message, get_var, KakError};

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
    Trim(trim::Options),
    Xargs(xargs::Options),
    Stdin(stdin::Options),
}

fn main() {
    if get_var("kak_command_fifo")
        .and(get_var("kak_response_fifo"))
        .is_err()
    {
        panic!("Environment variable kak_command_fifo and kak_response_fifo must be set");
    }

    let (msg, msg_details) = match run() {
        Ok(msg) => (msg, None),
        Err(e) => (e.to_string(), Some(e.details())),
    };

    if let Err(display_error) = display_message(&msg, msg_details.as_ref()) {
        // If there was an error sending the display message to kakoune, print it out
        eprintln!(
            "Error sending message '{msg:?}' (details: '{msg_details:?}') to kak: {display_error:?}"
        );
    }
}

fn run() -> Result<String, KakError> {
    let options =
        Cli::try_parse().map_err(|e| KakError::Parse(format!("Argument parse error: {e}")))?;

    match &options.command {
        Commands::Sort(o) => sort::sort(o),
        Commands::Shuf(o) => shuf::shuf(o),
        Commands::Uniq(o) => uniq::uniq(o),
        Commands::MathEval(o) => math_eval::math_eval(o),
        Commands::Trim(o) => trim::trim(o),
        Commands::Xargs(o) => xargs::xargs(o),
        Commands::Stdin(o) => stdin::stdin(o),
    }
}
