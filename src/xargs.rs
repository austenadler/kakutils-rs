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
    let mut child = Command::new("xargs")
        .arg("-0")
        .arg("--")
        .args(&options.args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to spawn child process");

    let mut stdin = child.stdin.take().expect("Failed to open stdin");
    let handle = std::thread::spawn(move || -> Result<(), KakMessage> {
        for s in get_selections_with_desc()? {
            eprintln!("Got selection {}", s.content);
            write!(stdin, "{}\0", s.content)?;
        }
        Ok(())
    });

    eprintln!("About t oreadvv");

    set_selections(
        BufReader::new(child.stdout.take().ok_or("Failed to get stdout")?)
            .split(b'\0')
            .map(|s| Ok(String::from_utf8_lossy(&s?).into_owned()))
            .collect::<Result<Vec<_>, KakMessage>>()?
            .iter(),
    )?;

    // Wait for the background process to exit
    // TODO: Do not use a string
    handle
        .join()
        .map_err(|_e| "Could not join background process")??;

    Ok("xargs selections".into())
}
