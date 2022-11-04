// use crate::utils;
use kakplugin::{
    get_register_selections, get_selections, get_selections_with_desc, set_selections_desc,
    types::Register, KakError,
};
use linked_hash_map::LinkedHashMap;
use linked_hash_set::LinkedHashSet;
use regex::Regex;
use std::{borrow::Cow, io::Write, str::FromStr};

const KAK_BUFFER_NAME: &str = "*kakplugin-set*";

#[derive(clap::StructOpt, Debug)]
pub struct Options {
    #[clap(
        min_values = 1,
        max_values = 3,
        allow_hyphen_values = true,
        help = "Register operation and operand. Empty register is current selection. Example: 'a-b' or '+b'"
    )]
    args: Vec<String>,

    #[clap(short, long, help = "Trim each selection before doing set operations")]
    skip_whitespace: bool,
    // #[clap(short, long)]
    #[clap(skip)]
    regex: Option<Regex>,
    // #[clap(short, long)]
    #[clap(skip)]
    ignore_case: bool,
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
    pub const fn to_char(&self) -> char {
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

pub fn set<'sel>(options: &'_ Options) -> Result<String, KakError> {
    // Get the actual operation we are performing
    let (left_register, operation, right_register) = parse_arguments(&options.args[..])?;

    // Get the selections for the left register and the right register, depending on the arguments
    // Underscore is a special case. We will treat it as the current selection
    let (left_selections, right_selections) = match (&left_register, &right_register) {
        (Register::Underscore, r) => {
            let l_selections = get_selections(None)?;
            let r_selections = get_register_selections(r)?;

            (l_selections, r_selections)
        }
        (l, Register::Underscore) => {
            let r_selections = get_selections(None)?;
            let l_selections = get_register_selections(l)?;

            (l_selections, r_selections)
        }
        (l, r) => {
            let l_selections = get_register_selections(l)?;
            let r_selections = get_register_selections(r)?;

            (l_selections, r_selections)
        }
    };

    // Get the frequency of each selection. The count does not matter as much as presence
    // Count is used only for compare
    let (left_ordered_counts, right_ordered_counts) = (
        to_ordered_counts(
            options,
            left_selections.iter().map(|s| s.as_ref()).collect(),
        ),
        to_ordered_counts(
            options,
            right_selections.iter().map(|s| s.as_ref()).collect(),
        ),
    );

    // Get an ordered set of every key for each register
    let (left_keys, right_keys) = (
        left_ordered_counts
            .keys()
            .map(|k| -> &str { k.as_ref() })
            .collect::<LinkedHashSet<&str>>(),
        right_ordered_counts
            .keys()
            .map(|k| -> &str { k.as_ref() })
            .collect::<LinkedHashSet<&str>>(),
    );

    // Run the actual set operation
    let result = key_set_operation(&operation, &left_keys, &right_keys);
    let num_modified = result.len();

    match &operation {
        Operation::Compare => compare(
            left_register,
            right_register,
            result,
            &left_ordered_counts,
            &right_ordered_counts,
        )?,
        Operation::Union => print_result(result)?,
        // Intersect/subtract will have at most the number of elements in the current selection
        // If the user operated on the current selection, and we can modify the selection descs inplace, do it
        Operation::Intersect | Operation::Subtract => {
            if left_register == Register::Underscore {
                // If the user asked for an intersection or subtraction from the current selection, we can update selection_descs only
                // For example (current selection) - (contents of register a) allows us to simply deselect some selections
                reduce_selections(options, result)?;
            } else {
                // The user asked for registers that *aren't* the current selection
                // This means either registers don't represent the current selection, or the current selection is on the other side
                print_result(result)?;
            }
        }
    }

    Ok(match &operation {
        Operation::Compare => format!("Compared {} selections", num_modified),
        op => format!(
            "{}{}{} returned {} selections",
            left_register.to_char(),
            op.to_char(),
            right_register.to_char(),
            num_modified
        ),
    })
}

/// Reduces selections to those that are in the `key_set_operation_result`
fn reduce_selections<'sel, 'a>(
    options: &Options,
    key_set_operation_result: LinkedHashSet<&'sel str>,
) -> Result<(), KakError> {
    // The registers should have been read in a draft context
    // So the current selection will be unmodified
    let selections_with_desc = get_selections_with_desc(None)?;

    set_selections_desc(selections_with_desc.into_iter().filter_map(|swd| {
        // Does not matter if the operation was - or &
        // Since key_set_operation_result contains elements that should be in the resulting set,
        // we can just use contains here
        let key = crate::utils::get_key(
            &swd.content,
            options.skip_whitespace,
            options.regex.as_ref(),
            options.ignore_case,
        );

        if key_set_operation_result.contains(key.as_ref()) {
            Some(swd.desc)
        } else {
            None
        }
    }))?;

    Ok(())
}

/// Writes the result of a set operation to a new kak buffer
fn print_result(key_set_operation_result: LinkedHashSet<&str>) -> Result<(), KakError> {
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
            edit -scratch '{}';
            execute-keys '%<a-R>_';
        }}"#,
        KAK_BUFFER_NAME
    )?;

    f.flush()?;

    Ok(())
}

/// Writes a comparison table to a new kak buffer
///
/// * `left_register` - Register of the left side
/// * `right_register` - Register of the right side
/// * `key_set_operation_result` - Set of selections after chosen operation
/// * `left_ordered_counts` - Map of ordered counts on `get_key` to frequency on the left side
/// * `right_ordered_counts` - Map of ordered counts on `get_key` to frequency on the right side
fn compare<'sel, 'a, 'b>(
    left_register: Register,
    right_register: Register,
    key_set_operation_result: LinkedHashSet<&'b str>,
    left_ordered_counts: &'b LinkedHashMap<Cow<'sel, str>, usize>,
    right_ordered_counts: &'b LinkedHashMap<Cow<'sel, str>, usize>,
) -> Result<(), KakError> {
    // Manually set selections so we don't have to allocate a string
    let mut f = kakplugin::open_command_fifo()?;

    // Send all of this into an evaluate-commands block
    write!(
        f,
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
        let left_count = left_ordered_counts.get(k as &str).unwrap_or(&0);
        let right_count = right_ordered_counts.get(k as &str).unwrap_or(&0);

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
            kakplugin::escape(k),
        )?;
    }

    write!(
        f,
        r#";
            edit -scratch '{}';
            execute-keys '%<a-R><a-;>3<a-W>L)<a-space>_vb';
        }}"#,
        KAK_BUFFER_NAME
    )?;

    f.flush()?;

    Ok(())
}

/// Counts frequency of unique selection contents, while preserving document order using a `LinkedHashMap`
///
/// # Returns
///
/// `LinkedHashMap` ordered by document order with `get_key(selection, ...)` as key and frequency of selection
fn to_ordered_counts<'sel>(
    options: &Options,
    selections: Vec<&'sel str>,
) -> LinkedHashMap<Cow<'sel, str>, usize> {
    let mut ret = LinkedHashMap::new();

    for i in selections {
        let key = crate::utils::get_key(
            &i,
            options.skip_whitespace,
            options.regex.as_ref(),
            options.ignore_case,
        );

        if key.is_empty() {
            // We don't want to even pretend to look at empty keys
            continue;
        }

        let entry: &mut usize = ret.entry(key).or_insert(0);
        *entry = entry.saturating_add(1);
    }
    ret
}

/// Performs an `Operation` on some set of keys
/// * `operation` - The operation to perform
/// * `left_keys` - The set on the left side of the operator
/// * `right_keys` - The set on the right side of the operator
fn key_set_operation<'sel>(
    operation: &Operation,
    left_keys: &LinkedHashSet<&'sel str>,
    right_keys: &LinkedHashSet<&'sel str>,
) -> LinkedHashSet<&'sel str> {
    match operation {
        Operation::Intersect => left_keys
            .intersection(right_keys)
            // .into_iter()
            .copied()
            .collect(),
        Operation::Subtract => left_keys.difference(right_keys).copied().collect(),
        Operation::Compare | Operation::Union => left_keys.union(right_keys).copied().collect(), // TODO: Symmetric difference?
    }
}

/// Parses the arguments used for set manipulation
///
/// Arguments can be given like `a-b`, `a - b`
fn parse_arguments(args: &[String]) -> Result<(Register, Operation, Register), KakError> {
    let args = if args.len() == 1 {
        // They gave us something like "a-b" or "c?d"
        args.iter()
            .flat_map(|s: &String| s.trim().chars())
            .map(String::from)
            .collect::<Vec<String>>()
    } else {
        // They gave us something like "a - b" or "c compare d"
        args.to_vec()
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
                (Err(_), Err(_)) => Err(KakError::Custom(
                    "One argument must be an operation".to_string(),
                )),
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
        _ => Err(KakError::Custom(
            "Invalid arguments to set command".to_string(),
        )),
    }?;

    if left_register == right_register {
        return Err(KakError::Custom(format!(
            "Registers passed are the same: '{}'",
            left_register.to_char()
        )));
    }

    Ok((left_register, middle, right_register))
}
