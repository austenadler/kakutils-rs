use evalexpr::{eval, Value};
use kakplugin::{get_selections, open_command_fifo, KakError};
use std::io::Write;

// TODO: Context for log() and others

#[derive(clap::StructOpt, Debug)]
pub struct Options;
pub fn math_eval(_options: &Options) -> Result<String, KakError> {
    let selections = get_selections()?;

    let mut f = open_command_fifo()?;
    write!(f, "reg '\"'")?;

    let mut err = None;
    let mut err_count: usize = 0;

    for i in selections.iter().map(|s| {
        // TODO: Do all of these need to be strings?
        match eval(s) {
            Ok(Value::Float(f)) => Some(f.to_string()),
            Ok(Value::Int(f)) => Some(f.to_string()),
            // TODO: Should this be none?
            Ok(_) => None,
            Err(e) => {
                eprintln!("Error: {:?}", e);
                if err.is_none() {
                    err = Some(e);
                    err_count = err_count.saturating_add(1);
                }
                None
            }
        }
    }) {
        // TODO: String allocation?
        let new_selection = i.map(|s| s.replace('\'', "''"));
        // .unwrap_or_else(|| "".to_string());
        write!(f, " '{}'", new_selection.as_deref().unwrap_or(""))?;
    }
    write!(f, " ; exec R;")?;
    f.flush()?;

    Ok(format!("MathEval {} selections", selections.len()))
}
