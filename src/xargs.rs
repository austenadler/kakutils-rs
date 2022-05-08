use crate::{get_selections_with_desc, set_selections, KakMessage};
use std::{
    io::{BufRead, BufReader, Write},
    process::{Command, Stdio},
};
#[derive(clap::StructOpt, Debug)]
pub struct Options {
    args: Vec<String>,
}
pub fn xargs(options: &Options) -> Result<KakMessage, KakMessage> {
    // let mut selections = get_selections()?;

    let mut child = Command::new("xargs")
        .arg("-0")
        .args(&options.args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to spawn child process");

    let mut stdin = child.stdin.take().expect("Failed to open stdin");
    let handle = std::thread::spawn(move || -> Result<(), KakMessage> {
        for s in get_selections_with_desc()? {
            write!(stdin, "{}\0", s.content)?;
            // stdin
            // .write_all(&.as_bytes())
            // .expect("Failed to write to stdin");
            // stdin.write_all(&[b'\0']).expect("Failed to write to stdin");
        }
        Ok(())
    });

    set_selections(BufReader::new(child.stdout.take().expect("Failed to get stdout")).split(b'\0'));

    // stdout.

    // set_selections(selections.iter())?;

    Ok(KakMessage(format!("Shuf  selections",), None))
}
