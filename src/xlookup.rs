use crate::utils::get_hash;
use evalexpr::{eval, Value};
use kakplugin::{
    get_selections, open_command_fifo, response, set_selections, types::Register, KakError,
    Selection,
};
use std::{
    borrow::Cow,
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
    #[clap(help = "Register with the lookup table", default_value = "\"")]
    register: Register,
}
pub fn xlookup(options: &Options) -> Result<String, KakError> {
    let lookup_table = build_lookuptable(kakplugin::reg(options.register, None)?)?;

    let selections = get_selections(None)?;

    let mut err_count: usize = 0;

    set_selections(selections.iter().map(|key| {
        lookup_table
            .get(&get_hash(key, false, None, false))
            .map_or_else(
                || {
                    eprintln!("Key '{key}' not found",);
                    err_count += 1;
                    Cow::Borrowed("")
                },
                |s| Cow::Owned(ToString::to_string(s)),
            )
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

fn build_lookuptable(mut selections: Vec<Selection>) -> Result<BTreeMap<u64, Selection>, KakError> {
    let mut iter = selections.array_chunks_mut();
    let ret = iter.try_fold(BTreeMap::new(), |mut acc, [key, value]| {
        match acc.entry(get_hash(key, false, None, false)) {
            Occupied(_) => Err(KakError::Custom(format!("Duplicate key '{key}'"))),
            Vacant(v) => {
                v.insert(value.clone());
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

#[cfg(test)]
mod tests {
    use super::*;
    macro_rules! blt {
        ($($x:expr),+ $(,)?) => {
            build_lookuptable(vec![$($x.to_string()),+])
        }
    }
    macro_rules! hsh {
        ($expr:expr) => {
            get_hash(&$expr.to_string(), false, None, false)
        };
    }
    #[test]
    fn test_build_lookuptable() {
        // Must be an even number
        assert!(blt!["1", "b", "c"].is_err());
        // Duplicate key
        assert!(blt!["1", "b", "2", "c", "2", "d"].is_err());
        // Valid
        assert!(blt!["1", "b", "2", "c"].is_ok());

        let lt = blt!["1", "b", "2", "c"].unwrap();
        assert_eq!(lt.get(&hsh!("1")), Some(&String::from("b")));
        assert_eq!(lt.get(&hsh!("2")), Some(&String::from("c")));
        assert_eq!(lt.get(&hsh!("3")), None);
    }
}
