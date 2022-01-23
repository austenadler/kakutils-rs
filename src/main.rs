#![warn(clippy::all, clippy::pedantic, clippy::nursery, clippy::cargo)]

use clap::Parser;
use regex::Regex;

#[derive(Parser)]
#[clap(about, version, author)]
struct Options {
    #[clap(short = 'S', long)]
    no_skip_whitespace: bool,
    // #[clap(short, long)]
    // debug: bool,
    #[clap(short, long, required = true)]
    regex: String,
    #[clap(multiple_occurrences = true, required = true)]
    selections: Vec<String>,
}

fn main() {
    if let Err(msg) = run() {
        output_message(&msg, false);
    }
}

fn output_message(msg: &str, debug: bool) {
    println!(
        "echo{}'{}';",
        if debug { " -debug" } else { " " },
        msg.replace("'", "''")
    );
}

fn run() -> Result<(), String> {
    let options = Options::try_parse().map_err(|e| format!("Error: {:?}", e))?;

    let replacement_re = options.regex;

    let re = Regex::new(&replacement_re)
        .map_err(|_| format!("Invalid regular expression: {}", replacement_re))?;

    let mut zipped = options
        .selections
        .iter()
        .skip(2)
        .zip(options.selections.iter().skip(2).map(|a| {
            let captures = re.captures(a)?;
            captures
                .get(1)
                .or_else(|| captures.get(0))
                .map(|m| m.as_str())
        }))
        .collect::<Vec<(&String, Option<&str>)>>();

    zipped.sort_by(|(a, a_key), (b, b_key)| {
        let a = a_key.unwrap_or(a);
        let b = b_key.unwrap_or(b);
        a.cmp(b)
    });

    print!("reg '\"'");
    for i in &zipped {
        let new_selection = i.0.replace("'", "''");
        print!(" '{}'", new_selection);
        // print!("{}\0", new_selection);
        // TODO: Allow debugging with -d
        // println!("\n\tSort key: {:?}", i.1);
    }
    print!(" ;");
    Ok(())
}
