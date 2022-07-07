use kakplugin::{get_selections_with_desc, set_selections, KakError};
use std::{
    io::{BufRead, BufReader, Write},
    process::{Command, Stdio},
};
#[derive(clap::StructOpt, Debug)]
pub struct Options {
    command: String,
    args: Vec<String>,
}
pub fn stdin(options: &Options) -> Result<String, KakError> {
    let mut child = Command::new(&options.command)
        .args(&options.args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to spawn child process");

    let mut child_stdin = child.stdin.take().expect("Failed to open stdin");
    let handle = std::thread::spawn(move || -> Result<(), KakError> {
        for s in get_selections_with_desc()? {
            write!(child_stdin, "{}\0", s.content)?;
        }
        Ok(())
    });

    set_selections(
        BufReader::new(child.stdout.take().expect("Failed to get stdout"))
            .split(b'\0')
            .map(|s| Ok(String::from_utf8_lossy(&s?).into_owned()))
            .collect::<Result<Vec<_>, KakError>>()?
            .iter(),
    )?;

    // Wait for the background process to exit
    handle
        .join()
        .map_err(|_e| KakError::Custom("Could not join background process".to_string()))??;

    Ok("stdin selections".into())
}
