use crate::{DatabaseSchema, Differ, MigrationPlanner, NodeDiff, NodeItem};
use anyhow::Result;
use std::{
    collections::{BTreeMap, HashSet},
    str::FromStr,
};

impl DatabaseSchema {
    pub fn plan(&self, other: &Self, verbose: bool) -> anyhow::Result<Vec<String>> {
        let mut migrations: Vec<String> = Vec::new();

        // diff on types
        migrations.extend(schema_diff(
            &self.composite_types,
            &other.composite_types,
            verbose,
        )?);
        // diff on tables
        migrations.extend(schema_diff(&self.tables, &other.tables, verbose)?);
        // diff on views
        migrations.extend(schema_diff(&self.views, &other.views, verbose)?);
        // diff on functions
        migrations.extend(schema_diff(&self.functions, &other.functions, verbose)?);

        Ok(migrations)
    }
}

fn schema_diff<T>(
    local: &BTreeMap<String, BTreeMap<String, T>>,
    remote: &BTreeMap<String, BTreeMap<String, T>>,
    verbose: bool,
) -> Result<Vec<String>>
where
    T: NodeItem + Clone + FromStr<Err = anyhow::Error> + PartialEq + Eq + 'static,
    NodeDiff<T>: MigrationPlanner<Migration = String>,
{
    let mut migrations: Vec<String> = Vec::new();
    let keys: HashSet<_> = local.keys().collect();
    let other_keys: HashSet<_> = remote.keys().collect();

    // process intersection
    let intersection = keys.intersection(&other_keys);
    for key in intersection {
        let local = local.get(*key).unwrap();
        let remote = remote.get(*key).unwrap();
        let keys: HashSet<_> = local.keys().collect();
        let other_keys: HashSet<_> = remote.keys().collect();
        let added = keys.difference(&other_keys);
        for key in added {
            let v = local.get(*key).unwrap().clone();
            let (id, t) = (v.id(), v.type_name());
            let diff = NodeDiff::with_new(v);
            if verbose && atty::is(atty::Stream::Stdout) {
                println!("{} {} is added:\n{}", t, id, diff.diff);
            }
            migrations.extend(diff.plan()?);
        }
        let removed = other_keys.difference(&keys);
        for key in removed {
            let v = remote.get(*key).unwrap().clone();
            let (id, t) = (v.id(), v.type_name());
            let diff = NodeDiff::with_old(v);
            if verbose && atty::is(atty::Stream::Stdout) {
                println!("{} {} is removed:\n{}", t, id, diff.diff);
            }
            migrations.extend(diff.plan()?);
        }
        let intersection = keys.intersection(&other_keys);
        for key in intersection {
            let local: T = local.get(*key).unwrap().to_string().parse()?;
            let remote: T = remote.get(*key).unwrap().to_string().parse()?;

            let diff = remote.diff(&local)?;
            if let Some(diff) = diff {
                if verbose && atty::is(atty::Stream::Stdout) {
                    println!(
                        "{} {} is changed:\n\n{}",
                        local.type_name(),
                        local.id(),
                        diff.diff
                    );
                }
                migrations.extend(diff.plan()?);
            }
        }
    }

    // process added
    let added = keys.difference(&other_keys);
    for key in added {
        migrations.push(format!("CREATE SCHEMA {}", key));
        for item in local.get(*key).unwrap().values() {
            let diff = NodeDiff::with_new(item.clone());
            if verbose && atty::is(atty::Stream::Stdout) {
                println!(
                    "{} {} is added:\n{}",
                    item.type_name(),
                    item.id(),
                    diff.diff
                );
            }
            migrations.extend(diff.plan()?);
        }
    }

    // process removed
    let removed = other_keys.difference(&keys);
    for key in removed {
        for item in remote.get(*key).unwrap().values() {
            let diff = NodeDiff::with_old(item.clone());
            if verbose && atty::is(atty::Stream::Stdout) {
                println!(
                    "{} {} is removed:\n{}",
                    item.type_name(),
                    item.id(),
                    diff.diff
                );
            }
            migrations.extend(diff.plan()?);
        }
        migrations.push(format!("DROP SCHEMA {}", key));
    }
    Ok(migrations)
}
