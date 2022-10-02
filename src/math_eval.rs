use evalexpr::{eval, Value};
use kakplugin::{get_selections, set_selections, KakError};
use std::borrow::Cow;

#[derive(clap::StructOpt, Debug)]
pub struct Options;
pub fn math_eval(_options: &Options) -> Result<String, KakError> {
    let mut err_count: usize = 0;

    let selections = get_selections(None)?;

    set_selections(selections.iter().map(|s| match eval(s) {
        Ok(Value::Float(f)) => Cow::Owned(f.to_string()),
        Ok(Value::Int(f)) => Cow::Owned(f.to_string()),
        Ok(_) => Cow::Borrowed(""),
        Err(e) => {
            eprintln!("Error: {:?}", e);
            err_count = err_count.saturating_add(1);
            // Set the selection to empty
            Cow::Borrowed("")
        }
    }))?;

    Ok(if err_count == 0 {
        format!("Processed {} selections", selections.len())
    } else {
        format!(
            "Processed {} selections ({} error{})",
            selections.len().saturating_sub(err_count),
            err_count,
            if err_count == 1 { "" } else { "s" }
        )
    })
}
