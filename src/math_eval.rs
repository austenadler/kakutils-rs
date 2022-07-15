use evalexpr::{eval, Value};
use kakplugin::{get_selections, open_command_fifo, set_selections, KakError, Selection};
use std::io::Write;

#[derive(clap::StructOpt, Debug)]
pub struct Options;
pub fn math_eval(_options: &Options) -> Result<String, KakError> {
    let mut err_count: usize = 0;

    let selections = get_selections()?;

    set_selections(selections.iter().map(|s| match eval(s) {
        Ok(Value::Float(f)) => f.to_string(),
        Ok(Value::Int(f)) => f.to_string(),
        Ok(_) => String::from(""),
        Err(e) => {
            eprintln!("Error: {:?}", e);
            err_count = err_count.saturating_add(1);
            // Set the selection to empty
            String::from("")
        }
    }))?;

    Ok(if err_count == 0 {
        format!("Processed {} selections", selections.len())
    } else {
        format!(
            "Processed {} selections ({} errors)",
            selections.len().saturating_sub(err_count),
            err_count
        )
    })
}
