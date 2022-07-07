// Enable clippy 'hard mode'
#![warn(clippy::all, clippy::pedantic, clippy::nursery)]
// Intended behavior (10_f64 as i32)
#![allow(clippy::cast_possible_truncation)]
// Cannot be fixed
#![allow(clippy::multiple_crate_versions)]
#![allow(clippy::struct_excessive_bools)]
// TODO: Remove
#![allow(dead_code, unused_imports)]

mod box_;
mod errors;
mod math_eval;
mod set;
mod shuf;
mod sort;
mod stdin;
mod trim;
mod uniq;
mod utils;
// mod xargs;
use clap::{Parser, Subcommand};
use kakplugin::{display_message, get_var, KakError};
use std::env;
use strum::VariantNames;

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

#[derive(Subcommand, Debug, strum::EnumVariantNames)]
#[strum(serialize_all = "kebab_case")]
enum Commands {
    #[clap(about = "Sorts selections based on content or content regex match")]
    Sort(sort::Options),
    #[clap(about = "Shuffle selections")]
    Shuf(shuf::Options),
    #[clap(about = "Find unique selections based on optional regex match")]
    Uniq(uniq::Options),
    #[clap(about = "Evaluate selections as a math expression", visible_aliases = &["bc", "eval"])]
    MathEval(math_eval::Options),
    #[clap(about = "Trim every selection")]
    Trim(trim::Options),
    #[clap(about = "Perform set operations on selections")]
    Set(set::Options),
    // #[clap(about = "")]
    // Xargs(xargs::Options),
    #[clap(about = "Pass each selection null terminated to a command")]
    Stdin(stdin::Options),
    #[clap(about = "Make boxes out of selections", visible_aliases = &["square"])]
    Box_(box_::Options),
}

fn main() {
    // First, check if we are just getting candidates to run the program. kak_command_fifo is not needed for this
    let args = env::args().collect::<Vec<_>>();
    if args.len() == 2 && args[1] == "shell-script-candidates" {
        match kakplugin::generate_shell_script_candidates(Commands::VARIANTS) {
            Err(e) => eprintln!("{e:?}"),
            Ok(()) => {}
        }
        return;
    }

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
    let options = Cli::try_parse().map_err(|e| KakError::Custom(format!("{e}")))?;

    match &options.command {
        Commands::Sort(o) => sort::sort(o),
        Commands::Shuf(o) => shuf::shuf(o),
        Commands::Uniq(o) => uniq::uniq(o),
        Commands::MathEval(o) => math_eval::math_eval(o),
        Commands::Trim(o) => trim::trim(o),
        Commands::Set(o) => set::set(o),
        // Commands::Xargs(o) => xargs::xargs(o),
        Commands::Stdin(o) => stdin::stdin(o),
        Commands::Box_(o) => box_::box_(o),
    }
}
