#![warn(clippy::all, clippy::pedantic, clippy::nursery, clippy::cargo)]

use regex::Regex;
use std::env;
fn main() {
    let args = env::args().collect::<Vec<String>>();
    assert!(args.len() < 2, "Usage: rust-selection-sort REGEX SEL1 [SEL2 ...]");

    let replacement_re = &args[1];

    let re = Regex::new(replacement_re).unwrap_or_else(|_| panic!(
        "Invalid regular expression: {}",
        replacement_re
    ));

    let mut zipped = args
        .iter()
        .skip(2)
        .zip(args.iter().skip(2).map(|a| {
            let captures = re.captures(a)?;
            captures.get(1).or_else(|| captures.get(0)).map(|m| m.as_str())
        }))
        .collect::<Vec<(&String, Option<&str>)>>();

    zipped.sort_by(|(a, a_key), (b, b_key)| {
        let a = a_key.unwrap_or(a);
        let b = b_key.unwrap_or(b);
        a.cmp(b)
    });

    for i in &zipped {
        print!("{}\0", i.0);
        // TODO: Allow debugging with -d
        // println!("\n\tSort key: {:?}", i.1);
    }
}
