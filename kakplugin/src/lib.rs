mod errors;
pub mod types;
pub use errors::KakError;
use std::{
    env,
    fs::{self, File, OpenOptions},
    io::{BufWriter, Write},
    str::FromStr,
};
use types::Register;
pub use types::{
    AnchorPosition, Selection, SelectionDesc, SelectionWithDesc, SelectionWithSubselections,
};

/// # Errors
///
/// Will return `Err` if command fifo could not be opened, read from, or written to
pub fn get_selections() -> Result<Vec<Selection>, KakError> {
    response("%val{selections}")
}

/// # Errors
///
/// Will return `Err` if command fifo could not be opened, read from, or written to
pub fn get_selections_desc() -> Result<Vec<SelectionDesc>, KakError> {
    response("%val{selections_desc}")?
        .iter()
        .map(|sd| SelectionDesc::from_str(sd))
        .collect::<Result<Vec<_>, KakError>>()
}

// pub fn get_selections_with_subselections(
//     register: &str,
// ) -> Result<Vec<SelectionWithSubselections>, KakError> {
//     // TODO: Escape register
//     let subselections = get_selections_with_desc()?;
//     exec(format!("\"{}z", register.replace('\'', "''")))?;
//     let selections = get_selections_with_desc()?;

//     for sel in selections {
//         for i in subselections {}
//     }
// }

/// # Errors
///
/// Will return `Err` if command fifo could not be opened, read from, or written to,
/// or if `selections.len() != selections_desc.len`
pub fn get_selections_with_desc() -> Result<Vec<SelectionWithDesc>, KakError> {
    let mut selections = get_selections()?;
    let selections_desc = get_selections_desc()?;

    if selections.len() != selections_desc.len() {
        return Err(KakError::KakResponse(format!(
            "When requesting selections (={}) and selections_desc (={}), their count did not match",
            selections.len(),
            selections_desc.len()
        )));
    }

    let min_selection = selections_desc.iter().min().ok_or_else(|| {
        KakError::KakResponse(format!(
            "Selections are empty, which should not be possible"
        ))
    })?;

    // Kakoune prints selections in file order, but prints selections_desc rotated based on current selection
    // Ex:
    //   [a] [b] (c) [d] where () is primary selection
    //   selections:      a b c d
    //   selections_desc: c d a b

    // Need to rotate selections by primary selection's position in the list
    match selections_desc.iter().position(|p| p == min_selection) {
        Some(i) => {
            selections.rotate_right(i);
        }
        None => {
            return Err(KakError::KakResponse(format!(
                "Primary selection {} not found in selection_desc list ({:#?})",
                min_selection, selections_desc
            )));
        }
    }

    selections
        .into_iter()
        .zip(selections_desc.into_iter())
        .map(|(content, desc)| Ok(SelectionWithDesc { content, desc }))
        .collect::<Result<Vec<_>, _>>()
}

/// # Errors
///
/// Will return `Err` if command fifo could not be opened, read from, or written to
pub fn set_selections<'a, I, S: 'a + ?Sized>(selections: I) -> Result<(), KakError>
where
    I: IntoIterator<Item = &'a S>,
    S: AsRef<str>,
{
    let mut f = open_command_fifo()?;
    write!(f, "set-register '\"'")?;
    for i in selections {
        write!(f, " '{}'", escape(i))?;
    }
    write!(f, "; execute-keys R;")?;
    f.flush()?;
    Ok(())
}

/// # Errors
///
/// Will return `Err` if command fifo could not be opened, read from, or written to
pub fn set_selections_desc<'a, I>(selections: I) -> Result<(), KakError>
where
    I: IntoIterator<Item = &'a SelectionDesc>,
{
    let mut f = open_command_fifo()?;
    write!(f, "select")?;
    for i in selections {
        write!(f, " {}", i)?;
    }
    write!(f, ";")?;
    f.flush()?;
    Ok(())
}

/// # Errors
///
/// Will return `Err` if command fifo could not be opened, read from, or written to
pub fn display_message<S: AsRef<str>>(
    message: S,
    debug_message: Option<S>,
) -> Result<(), KakError> {
    let msg_str = escape(message);
    {
        let mut f = open_command_fifo()?;

        write!(f, "echo '{}';", msg_str)?;
        write!(f, "echo -debug '{}';", msg_str)?;

        if let Some(debug_msg_str) = &debug_message.as_ref() {
            write!(f, "echo -debug '{}';", escape(debug_msg_str))?;
        }
        f.flush()?;
    }
    Ok(())
}

pub fn escape<S: AsRef<str>>(s: S) -> String {
    s.as_ref().replace('\'', "''")
}

/// # Errors
///
/// Will return `Err` if command fifo could not be opened or written to
pub fn cmd(cmd: &str) -> Result<(), KakError> {
    let mut f = open_command_fifo()?;

    write!(f, "{};", cmd)?;
    f.flush().map_err(Into::into)
}

pub fn restore_register(r: &Register) -> Result<(), KakError> {
    cmd(&format!("execute-keys '\"{}z'", r.kak_escaped()))
}

/// # Errors
///
/// Will return `Err` if command fifo could not be opened or written to
pub fn response(msg: &str) -> Result<Vec<String>, KakError> {
    cmd(&format!(
        "echo -quoting shell -to-file {} -- {msg}",
        get_var("kak_response_fifo")?
    ))?;

    let selections = shellwords::split(&fs::read_to_string(&get_var("kak_response_fifo")?)?)?;

    Ok(selections)
}

/// # Errors
///
/// Will return `Err` if command fifo could not be opened
pub fn open_command_fifo() -> Result<BufWriter<File>, KakError> {
    OpenOptions::new()
        .write(true)
        .append(true)
        .open(&get_var("kak_command_fifo")?)
        .map(BufWriter::new)
        .map_err(Into::into)
}

/// # Errors
///
/// Will return `Err` if requested environment variable is not unicode or not present
pub fn get_var(var_name: &str) -> Result<String, KakError> {
    env::var(var_name).map_err(|e| match e {
        env::VarError::NotPresent => {
            KakError::EnvVarNotSet(format!("Env var {} is not defined", var_name))
        }
        env::VarError::NotUnicode(_) => {
            KakError::EnvVarUnicode(format!("Env var {} is not unicode", var_name))
        }
    })
}
