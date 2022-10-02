mod errors;
pub mod types;
pub use errors::KakError;
pub use shell_words::ParseError;
use std::{
    borrow::Cow,
    env,
    fmt::Display,
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
pub fn get_selections(keys: Option<&'_ str>) -> Result<Vec<Selection>, KakError> {
    response("%val{selections}", keys)
}

/// # Errors
///
/// Will return `Err` if command fifo could not be opened, read from, or written to
// TODO: Use AsRef
pub fn get_selections_desc<S>(keys: Option<S>) -> Result<Vec<SelectionDesc>, KakError>
where
    S: AsRef<str>,
{
    let mut ret = response("%val{selections_desc}", keys.as_ref())?
        .iter()
        .map(|sd| SelectionDesc::from_str(sd).map(|x| x.sort()))
        .collect::<Result<Vec<_>, KakError>>()?;
    ret.sort();
    Ok(ret)
}

/// # Errors
///
/// Will return `Err` if command fifo could not be opened, read from, or written to
// TODO: Use AsRef
pub fn get_selections_desc_unordered<S>(keys: Option<S>) -> Result<Vec<SelectionDesc>, KakError>
where
    S: AsRef<str>,
{
    response("%val{selections_desc}", keys.as_ref())?
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

/// Return a vec of SelectionWithDesc. The returned vec is in order according to SelectionDesc
///
/// For example, if your primary selection is selection 2 of 3, the returned vec's order will be selection 2, 3, then 1
///
/// # Errors
///
/// Will return `Err` if command fifo could not be opened, read from, or written to,
/// or if `selections.len() != selections_desc.len`
pub fn get_selections_with_desc(keys: Option<&'_ str>) -> Result<Vec<SelectionWithDesc>, KakError> {
    let mut selections = get_selections(keys)?;
    let selections_desc = get_selections_desc_unordered(keys)?;

    if selections.len() != selections_desc.len() {
        return Err(KakError::KakResponse(format!(
            "When requesting selections (={}) and selections_desc (={}), their count did not match",
            selections.len(),
            selections_desc.len()
        )));
    }

    let min_selection = selections_desc.iter().min().ok_or_else(|| {
        KakError::KakResponse("Selections are empty, which should not be possible".to_string())
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

/// Return a vec of SelectionWithDesc, sorted in file (SelectionDesc) order
///
/// For example, the returned vec's order will be selection 1, 2, then 3 regardless of the primary selection
pub fn get_selections_with_desc_ordered(
    keys: Option<&'_ str>,
) -> Result<Vec<SelectionWithDesc>, KakError> {
    let mut ret = get_selections_with_desc(keys)?;
    ret.sort_by_key(|s| s.desc.sort());
    Ok(ret)
}

/// # Errors
///
/// Will return `Err` if command fifo could not be opened, read from, or written to
pub fn set_selections<'a, I, S: 'a>(selections: I) -> Result<(), KakError>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str> + Clone + Display,
{
    let mut selections_iter = selections.into_iter().peekable();
    if selections_iter.peek().is_none() {
        return Err(KakError::SetEmptySelections);
    }

    let mut f = open_command_fifo()?;
    write!(f, "set-register '\"'")?;
    for i in selections_iter {
        write!(f, " '{}'", escape(i.as_ref()))?;
    }
    write!(f, "; execute-keys R;")?;
    f.flush()?;
    Ok(())
}

/// # Errors
///
/// Will return `Err` if command fifo could not be opened, read from, or written to
pub fn set_selections_desc<'a, I, SD: 'a + Display>(selections: I) -> Result<(), KakError>
where
    I: IntoIterator<Item = SD>,
    SD: AsRef<SelectionDesc>,
{
    let mut selections_iter = selections.into_iter().peekable();
    if selections_iter.peek().is_none() {
        return Err(KakError::SetEmptySelections);
    }

    let mut f = open_command_fifo()?;
    write!(f, "select")?;
    for i in selections_iter {
        write!(f, " {}", i)?;
    }
    write!(f, ";")?;
    f.flush()?;
    Ok(())
}

/// # Errors
///
/// Will return `Err` if command fifo could not be opened, read from, or written to
pub fn display_message<S: AsRef<str> + Clone + Display>(
    message: S,
    debug_message: Option<S>,
) -> Result<(), KakError> {
    let msg_str = escape(message.as_ref());
    {
        let mut f = open_command_fifo()?;

        write!(f, "echo '{}';", msg_str)?;
        write!(f, "echo -debug '{}';", msg_str)?;

        if let Some(debug_msg_str) = &debug_message.as_ref() {
            write!(f, "echo -debug '{}';", escape(debug_msg_str.as_ref()))?;
        }
        f.flush()?;
    }
    Ok(())
}

/// Escapes a string to be sent to kak by replacing single tick with two single tics
///
/// # Examples
///
/// ```
/// use kakplugin::escape;
/// use std::borrow::Cow;
///
/// assert_eq!(escape("abcd"), "abcd");
/// assert_eq!(escape("'ab\\cd'"), "''ab\\cd''");
///
/// // Will not reallocate for
/// assert!(matches!(escape("abcd"), Cow::Borrowed(_)));
/// assert!(matches!(escape("ab\\nc\nd"), Cow::Borrowed(_)));
/// assert!(matches!(escape("ab'cd"), Cow::Owned(_)));
/// ```
pub fn escape(s: &str) -> Cow<str> {
    if s.contains('\'') {
        Cow::Owned(s.replace('\'', "''"))
    } else {
        Cow::Borrowed(s)
    }
}

/// # Errors
///
/// Will return `Err` if command fifo could not be opened or written to
pub fn cmd<S>(cmd: S) -> Result<(), KakError>
where
    S: AsRef<str>,
{
    let mut f = open_command_fifo()?;

    write!(f, "{};", cmd.as_ref())?;
    f.flush().map_err(Into::into)
}

pub fn restore_register<R>(r: R) -> Result<(), KakError>
where
    R: AsRef<Register>,
{
    cmd(&format!("execute-keys '\"{}z'", r.as_ref().kak_escaped()))
}

/// # Errors
///
/// Will return `Err` if command fifo could not be opened or written to
pub fn response<S1, S2>(msg: S1, keys: Option<S2>) -> Result<Vec<String>, KakError>
where
    S1: AsRef<str>,
    S2: AsRef<str>,
{
    let response_fifo = get_var("kak_response_fifo")?;

    cmd(match keys.as_ref() {
        None => format!(
            "echo -quoting shell -to-file {response_fifo} -- {}",
            msg.as_ref()
        ),
        Some(keys) => format!(
            r#"evaluate-commands -draft %{{
    execute-keys '{}';
    echo -quoting shell -to-file {response_fifo} -- {};
}}"#,
            escape(keys.as_ref()),
            msg.as_ref()
        ),
    })?;

    Ok(shell_words::split(&fs::read_to_string(&get_var(
        "kak_response_fifo",
    )?)?)?)
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
pub fn get_var<S>(var_name: S) -> Result<String, KakError>
where
    S: AsRef<str>,
{
    env::var(var_name.as_ref()).map_err(|e| match e {
        env::VarError::NotPresent => {
            KakError::EnvVarNotSet(format!("Env var {} is not defined", var_name.as_ref()))
        }
        env::VarError::NotUnicode(_) => {
            KakError::EnvVarUnicode(format!("Env var {} is not unicode", var_name.as_ref()))
        }
    })
}

/// Prints a list of shell script candidates for kakoune to ingest
pub fn generate_shell_script_candidates<S>(variants: &[S]) -> Result<(), KakError>
where
    S: AsRef<str>,
{
    let token_to_complete = get_var("kak_token_to_complete")?.parse::<u8>()?;

    match token_to_complete {
        0 => {
            for v in variants {
                println!("{}", v.as_ref());
            }
        }
        1_u8..=u8::MAX => {
            // We can't see which command was selected, so none of these will do anything
        }
    }

    Ok(())
}
