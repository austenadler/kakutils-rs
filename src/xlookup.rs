use crate::utils::get_hash;
use evalexpr::{eval, Value};
use kakplugin::{
    get_selections, open_command_fifo, set_selections, types::Register, KakError, Selection,
};
use std::{
    collections::{
        btree_map::Entry::{Occupied, Vacant},
        hash_map::DefaultHasher,
        BTreeMap,
    },
    hash::{Hash, Hasher},
    io::Write,
};

#[derive(clap::StructOpt, Debug)]
pub struct Options {
    #[clap(help = "Register with the lookup table")]
    register: Register,
}
pub fn xlookup(options: &Options) -> Result<String, KakError> {
    let lookup_table = build_lookuptable(options.register)?;
    eprintln!("Lookup table: {lookup_table:#?}");

    let selections = get_selections(None)?;

    let mut err_count: usize = 0;

    set_selections(selections.iter().map(|key| {
        match lookup_table.get(&get_hash(&key, false, None, false)) {
            Some(v) => v.to_string(),
            None => {
                eprintln!(
                    "Nothing for '{key}' ({})",
                    get_hash(&key, false, None, false)
                );
                err_count += 1;
                String::from("")
            }
        }
    }))?;

    Ok(if err_count == 0 {
        format!("Xlookup {} selections", selections.len())
    } else {
        format!(
            "Xlookup {} selections ({} error{})",
            selections.len().saturating_sub(err_count),
            err_count,
            if err_count == 1 { "" } else { "s" }
        )
    })
}

pub fn build_lookuptable(reg: Register) -> Result<BTreeMap<u64, Selection>, KakError> {
    let mut selections = get_selections(Some(&format!("\"{reg}z")))?;
    let mut iter = selections.array_chunks_mut();
    let ret = iter.try_fold(BTreeMap::new(), |mut acc, [key, value]| {
        match acc.entry(get_hash(&key, false, None, false)) {
            Occupied(_) => Err(KakError::Custom(format!("Duplicate key '{key}'"))),
            Vacant(v) => {
                v.insert(value.to_owned());
                Ok(acc)
            }
        }
    })?;

    if !iter.into_remainder().is_empty() {
        Err(KakError::CustomStatic("Odd number of selections"))
    } else if ret.is_empty() {
        Err(KakError::CustomStatic("No selections"))
    } else {
        Ok(ret)
    }
}
