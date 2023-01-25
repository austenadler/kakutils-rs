// Enable clippy 'hard mode'
#![warn(clippy::all, clippy::pedantic, clippy::nursery)]
// Intended behavior (10_f64 as i32)
#![allow(clippy::cast_possible_truncation)]
// Cannot be fixed
#![allow(clippy::multiple_crate_versions)]
#![allow(clippy::struct_excessive_bools)]
#![feature(slice_group_by)]
#![feature(slice_take)]
#![feature(array_chunks)]

mod box_;
mod rev;
mod errors;
mod incr;
mod invert;
mod math_eval;
mod pad;
mod set;
mod shuf;
mod sort;
mod trim;
mod uniq;
mod utils;
mod xargs;
mod xlookup;
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
    #[clap(about = "Invert all selections", visible_aliases = &["inv", "inverse"])]
    Invert(invert::Options),
    #[clap(about = "Evaluate selections as a math expression", visible_aliases = &["bc", "eval"])]
    MathEval(math_eval::Options),
    #[clap(about = "Pad all selections by some specifier")]
    Pad(pad::Options),
    #[clap(about = "Trim every selection")]
    Trim(trim::Options),
    #[clap(about = "Perform set operations on selections")]
    Set(set::Options),
    #[clap(about = "Pass each selection null terminated to a command", visible_aliases = &["stdin"])]
    Xargs(xargs::Options),
    #[clap(about = "Make boxes out of selections", visible_aliases = &["square"])]
    Box_(box_::Options),
    #[clap(about = "Map selections based on a register", visible_aliases = &["vlookup"])]
    Xlookup(xlookup::Options),
    #[clap(about = "Increment selections")]
    Decr(incr::Options),
    #[clap(about = "Decrement selections")]
    Incr(incr::Options),
    #[clap(about = "Reverse selectinos")]
    Rev(rev::Options)
}

fn main() {
    // First, check if we are just getting candidates to run the program. kak_command_fifo is not needed for this
    let args = env::args().collect::<Vec<_>>();
    if args.len() >= 2 && args[1] == "shell-script-candidates" {
        if let Err(e) = kakplugin::generate_shell_script_candidates(Commands::VARIANTS) {
            eprintln!("{e:?}");
        }
        return;
    }

    // This will be required for all subcommands from here on
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
        Commands::Invert(o) => invert::invert(o),
        Commands::MathEval(o) => math_eval::math_eval(o),
        Commands::Pad(o) => pad::pad(o),
        Commands::Trim(o) => trim::trim(o),
        Commands::Set(o) => set::set(o),
        Commands::Xargs(o) => xargs::xargs(o),
        Commands::Box_(o) => box_::box_(o),
        Commands::Xlookup(o) => xlookup::xlookup(o),
        Commands::Incr(o) => incr::incr(o, true),
        Commands::Decr(o) => incr::incr(o, false),
        Commands::Rev(o) => rev::rev(o),
    }
}
