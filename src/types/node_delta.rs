use crate::{DeltaItem, NodeDelta};
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt,
};

impl<T, Item> NodeDelta<T>
where
    T: Clone + Ord + DeltaItem<SqlNode = Item> + fmt::Debug,
{
    pub fn create(old: BTreeMap<&String, &T>, new: BTreeMap<&String, &T>) -> NodeDelta<T> {
        let mut delta = NodeDelta::default();

        let old_keys: BTreeSet<_> = old.keys().collect();
        let new_keys: BTreeSet<_> = new.keys().collect();
        let added = new_keys.difference(&old_keys);
        let removed = old_keys.difference(&new_keys);
        let might_changed = old_keys.intersection(&new_keys);

        for key in added {
            delta.added.insert(new[*key].clone());
        }

        for key in removed {
            delta.removed.insert(old[*key].clone());
        }

        for key in might_changed {
            let old_priv = old[*key];
            let new_priv = new[*key];
            if old_priv.to_string() != new_priv.to_string() {
                delta.changed.insert((old_priv.clone(), new_priv.clone()));
            }
        }

        delta
    }

    pub fn plan(self, item: &Item) -> anyhow::Result<Vec<String>> {
        let mut migrations = Vec::new();

        let mut is_rename = false;
        // check if it is a case for rename
        if self.added.len() == 1 && self.removed.len() == 1 {
            let added = self.added.iter().next().unwrap();
            let removed = self.removed.iter().next().unwrap();
            let result = removed.to_owned().rename(item, added.to_owned())?;
            if !result.is_empty() {
                migrations.extend(result);
                is_rename = true;
            }
        }

        if !is_rename {
            for removed in self.removed {
                let sqls = removed.drop(item)?;
                migrations.extend(sqls);
            }

            for added in self.added {
                let sqls = added.create(item)?;
                migrations.extend(sqls);
            }
        }

        for (v1, v2) in self.changed {
            let sqls = v1.alter(item, v2)?;
            migrations.extend(sqls);
        }
        Ok(migrations)
    }
}

impl<T> Default for NodeDelta<T> {
    fn default() -> Self {
        Self {
            added: BTreeSet::new(),
            removed: BTreeSet::new(),
            changed: BTreeSet::new(),
        }
    }
}
