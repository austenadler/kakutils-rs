use kakplugin::{get_selections_with_desc, set_selections, KakError};
use std::{
    io::{BufRead, BufReader, Write},
    process::{Command, Stdio},
};
#[derive(clap::StructOpt, Debug)]
pub struct Options {
    args: Vec<String>,
}
pub fn xargs(options: &Options) -> Result<String, KakError> {
    let mut child = Command::new("xargs")
        .arg("-0")
        .arg("--")
        .args(&options.args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to spawn child process");

    let mut stdin = child.stdin.take().expect("Failed to open stdin");
    let handle = std::thread::spawn(move || -> Result<(), KakError> {
        for s in get_selections_with_desc()? {
            eprintln!("Got selection {}", s.content);
            write!(stdin, "{}\0", s.content)?;
        }
        Ok(())
    });

    eprintln!("About t oreadvv");

    set_selections(
        BufReader::new(
            child
                .stdout
                .take()
                .ok_or(KakError::Custom("Failed to get stdout".to_string()))?,
        )
        .split(b'\0')
        .map(|s| Ok(String::from_utf8_lossy(&s?).into_owned()))
        .collect::<Result<Vec<_>, KakError>>()?
        .iter(),
    )?;

    // Wait for the background process to exit
    // TODO: Do not use a string
    handle
        .join()
        .map_err(|_e| KakError::Custom("Could not join background process".to_string()))??;

    Ok("xargs selections".into())
}
