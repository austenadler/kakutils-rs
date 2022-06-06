// use crate::utils;
use clap::ArgEnum;
use kakplugin::{
    get_selections, get_selections_with_desc, set_selections, set_selections_desc, types::Register,
    KakError, Selection, SelectionWithDesc,
};
use linked_hash_map::LinkedHashMap;
use linked_hash_set::LinkedHashSet;
use regex::Regex;
use std::{collections::HashSet, io::Write, str::FromStr};

#[derive(clap::StructOpt, Debug)]
pub struct Options {
    #[clap(min_values = 1, max_values = 3)]
    args: Vec<String>,

    #[clap(short = 'T')]
    no_trim: bool,
    // #[clap(short, long)]
    // regex: Option<Regex>,
    // #[clap(short, long)]
    // ignore_case: bool,
    // #[clap(short = 'S', long)]
    // no_skip_whitespace: bool,
}

#[derive(Clone, Debug)]
enum Operation {
    Intersect,
    Subtract,
    Union,
    Compare,
}

impl Operation {
    pub fn to_char(&self) -> char {
        match self {
            Self::Intersect => '&',
            Self::Subtract => '-',
            Self::Union => '+',
            Self::Compare => '?',
        }
    }
}

impl FromStr for Operation {
    type Err = KakError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "intersect" | "and" | "&" => Ok(Self::Intersect),
            "subtract" | "not" | "minus" | "-" | "\\" => Ok(Self::Subtract),
            "union" | "or" | "plus" | "+" => Ok(Self::Union),
            "compare" | "cmp" | "?" | "=" => Ok(Self::Compare),
            _ => Err(KakError::Parse(format!(
                "Set operation '{s}' could not be parsed"
            ))),
        }
    }
}

pub fn set(options: &Options) -> Result<String, KakError> {
    let (left_register, operation, right_register) = parse_arguments(&options.args[..])?;

    // Underscore is a special case. We will treat it as the current selection
    let (left_selections, right_selections) = match (&left_register, &right_register) {
        (Register::Underscore, r) => {
            let l_selections = get_selections()?;
            kakplugin::restore_register(r)?;
            let r_selections = get_selections()?;

            (l_selections, r_selections)
        }
        (l, Register::Underscore) => {
            let r_selections = get_selections()?;
            kakplugin::restore_register(l)?;
            let l_selections = get_selections()?;

            (l_selections, r_selections)
        }
        (l, r) => {
            kakplugin::restore_register(l)?;
            let l_selections = get_selections()?;

            kakplugin::restore_register(r)?;
            let r_selections = get_selections()?;
            (l_selections, r_selections)
        }
    };

    let (left_ordered_counts, right_ordered_counts) = (
        to_ordered_counts(options, left_selections),
        to_ordered_counts(options, right_selections),
    );
    let (left_keys, right_keys) = (
        left_ordered_counts
            .keys()
            .collect::<LinkedHashSet<&Selection>>(),
        right_ordered_counts
            .keys()
            .collect::<LinkedHashSet<&Selection>>(),
    );

    let result = key_set_operation(&operation, &left_keys, &right_keys);

    match &operation {
        Operation::Compare => compare(
            &left_register,
            &right_register,
            &result,
            &left_ordered_counts,
            &right_ordered_counts,
        )?,
        Operation::Intersect | Operation::Subtract | Operation::Union => print_result(&result)?,
    }

    Ok(match &operation {
        Operation::Compare => format!("Compared {} selections", result.len()),
        op => format!(
            "{}{}{} returned {} selections",
            left_register.to_char(),
            op.to_char(),
            right_register.to_char(),
            result.len()
        ),
    })
}

fn print_result(key_set_operation_result: &LinkedHashSet<&Selection>) -> Result<(), KakError> {
    // Manually set selections so we don't have to allocate a string
    let mut f = kakplugin::open_command_fifo()?;

    // Send all of this into an evaluate-commands block
    //  -save-regs '"'
    write!(
        f,
        r#"evaluate-commands %{{
                set-register '"'"#
    )?;

    for k in key_set_operation_result {
        write!(f, " '{}\n'", kakplugin::escape(k))?;
    }

    write!(
        f,
        r#";
            edit -scratch '*kakplugin-set*';
            execute-keys '%<a-R>_';
        }}"#
    )?;

    f.flush()?;

    Ok(())
}

fn compare(
    left_register: &Register,
    right_register: &Register,
    key_set_operation_result: &LinkedHashSet<&Selection>,
    left_ordered_counts: &LinkedHashMap<Selection, usize>,
    right_ordered_counts: &LinkedHashMap<Selection, usize>,
) -> Result<(), KakError> {
    // Manually set selections so we don't have to allocate a string
    let mut f = kakplugin::open_command_fifo()?;

    // Send all of this into an evaluate-commands block
    write!(
        f,
        // -save-regs '"'
        r#"evaluate-commands -save-regs '"' %{{
                set-register '"'"#
    )?;

    write!(
        f,
        " '?\t{}\t{}\tselection\n'",
        left_register.to_char(),
        right_register.to_char()
    )?;

    for k in key_set_operation_result {
        let left_count = left_ordered_counts.get(&k as &str).unwrap_or(&0);
        let right_count = right_ordered_counts.get(&k as &str).unwrap_or(&0);

        write!(
            f,
            " '{}\t{}\t{}\t{}\n'",
            match (*left_count == 0, *right_count == 0) {
                (true, true) => "?",
                (true, false) => ">",
                (false, true) => "<",
                (false, false) => "=",
            },
            left_count,
            right_count,
            // TODO: Do we want to escape the \n to \\n?
            // kakplugin::escape(k.replace('\n', "\\n")),
            kakplugin::escape(k),
        )?;
    }

    write!(
        f,
        r#";
            edit -scratch '*kakplugin-set*';
            execute-keys '%<a-R><a-;>3<a-W>L)<a-space>_vb';
        }}"#
    )?;

    f.flush()?;

    Ok(())
}

fn to_ordered_counts(options: &Options, sels: Vec<Selection>) -> LinkedHashMap<Selection, usize> {
    let mut ret = LinkedHashMap::new();

    for i in sels.into_iter() {
        let key = if options.no_trim {
            i
        } else {
            i.trim().to_string()
        };

        if key.is_empty() {
            // We don't want to even pretend to look at empty keys
            continue;
        }

        let entry: &mut usize = ret.entry(key).or_insert(0);
        *entry = entry.saturating_add(1);
    }
    ret
}

fn key_set_operation<'a>(
    operation: &Operation,
    left_keys: &LinkedHashSet<&'a Selection>,
    right_keys: &LinkedHashSet<&'a Selection>,
) -> LinkedHashSet<&'a Selection> {
    match operation {
        Operation::Intersect => left_keys
            .intersection(&right_keys)
            // .into_iter()
            // TODO: Remove this
            .cloned()
            .collect(),
        Operation::Subtract => left_keys
            .difference(&right_keys)
            .into_iter()
            // TODO: Remove this
            .cloned()
            .collect(),
        Operation::Compare | Operation::Union => left_keys
            .union(&right_keys)
            .into_iter()
            // TODO: Remove this
            .cloned()
            .collect(),
        // TODO: Symmetric difference?
    }
}

fn parse_arguments(args: &[String]) -> Result<(Register, Operation, Register), KakError> {
    let args = if args.len() == 1 {
        // They gave us something like "a-b" or "c?d"
        args.iter()
            .flat_map(|s: &String| s.trim().chars())
            .map(|c| String::from(c))
            .collect::<Vec<String>>()
    } else {
        // They gave us something like "a - b" or "c compare d"
        args.iter().cloned().collect()
    };
    let (left_register, middle, right_register) = match &args[..] {
        [l, r] => {
            // They only gave us two arguments like "- a" or "b -"
            match (Operation::from_str(l), Operation::from_str(r)) {
                // If the operation is on the left, then the _ register is the leftmost one
                (Ok(o), Err(_)) => Ok((Register::Underscore, o, Register::from_str(r)?)),
                // If the operation is on the right, then the _ register is the rightmost one
                (Err(_), Ok(o)) => Ok((Register::from_str(l)?, o, Register::Underscore)),
                (Ok(_), Ok(_)) => Err(KakError::Custom(format!(
                    "Arguments '{l}' and '{r}' cannot both be operations"
                ))),
                (Err(_), Err(_)) => Err(KakError::Custom(format!(
                    "One argument must be an operation"
                ))),
            }
        }
        [l, middle, r] => {
            // They gave us three arguments like "a - b" or "_ + a"
            Ok((
                Register::from_str(l)?,
                Operation::from_str(middle)?,
                Register::from_str(r)?,
            ))
        }
        _ => Err(KakError::Custom(format!(
            "Invalid arguments to set command"
        ))),
    }?;

    if left_register == right_register {
        return Err(KakError::Custom(format!(
            "Registers passed are the same: '{}'",
            left_register.to_char()
        )));
    }

    Ok((left_register, middle, right_register))
}
