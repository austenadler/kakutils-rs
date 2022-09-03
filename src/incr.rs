use evalexpr::{eval, Value};
use kakplugin::{get_selections, open_command_fifo, set_selections, KakError, Selection};
use std::io::Write;

#[derive(clap::StructOpt, Debug)]
pub struct Options {
    #[clap(index = 1, help = "Amount to increment/decrement", default_value = "1")]
    amount: isize,
}

pub fn incr(options: &Options, should_increment: bool) -> Result<String, KakError> {
    let mut err_count: usize = 0;

    let selections = get_selections(Some("_"))?;

    set_selections(selections.iter().map(|s| {
        match eval(&format!(
            "{s}{}{}",
            if should_increment { "+" } else { "-" },
            options.amount
        )) {
            Ok(Value::Float(f)) => f.to_string(),
            Ok(Value::Int(f)) => f.to_string(),
            Ok(_) => String::from(""),
            Err(e) => {
                eprintln!("Error: {:?}", e);
                err_count = err_count.saturating_add(1);
                // Set the selection to empty
                String::from("")
            }
        }
    }))?;

    Ok(if err_count == 0 {
        format!("{} {} selections by {}",if options.should_increment {"Incr"} else {"Decr"}, selections.len(), options.amount)
    } else {
        format!(
            "{} {} selections by {} ({} errors)",if options.should_increment {"Incr"} else {"Decr"},
            selections.len().saturating_sub(err_count),
            options.amount,
            err_count,
        )
    })
}
