use kakplugin::{get_selections_with_desc, set_selections_failable, KakError};
use std::{
    io::{BufRead, BufReader, Write},
    process::{Command, Stdio},
};
#[derive(clap::Args, Debug)]
pub struct Options {
    #[clap()]
    command: String,
    #[clap(allow_hyphen_values = true)]
    args: Vec<String>,
}
pub fn xargs(options: &Options) -> Result<String, KakError> {
    let mut child = Command::new(&options.command)
        .args(&options.args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to spawn child process");

    let mut child_stdin = child.stdin.take().expect("Failed to open stdin");
    let handle = std::thread::spawn(move || -> Result<(), KakError> {
        for s in get_selections_with_desc(None)? {
            write!(child_stdin, "{}\0", s.content)?;
        }
        Ok(())
    });

    let set_selections_result = set_selections_failable(
        BufReader::new(child.stdout.take().expect("Failed to get stdout"))
            .split(b'\0')
            // TODO: Support non-utf8?
            .map(|s| -> Result<_, KakError> { Ok(String::from_utf8(s?)?) }),
    );

    // Wait for the background process to exit
    // Return its error (if there is one) first
    handle
        .join()
        .map_err(|_e| KakError::Custom("Could not join background process".to_string()))??;

    // Now print any errors
    let num_set = set_selections_result?;

    Ok(format!(
        "Set {} selections from {}",
        num_set, options.command
    ))
}
