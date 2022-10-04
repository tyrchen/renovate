use crate::NodeDelta;
use pg_query::NodeEnum;
use std::collections::{BTreeMap, BTreeSet};

impl<T> NodeDelta<T>
where
    T: Clone + Ord,
{
    pub fn calculate(old: &BTreeMap<String, T>, new: &BTreeMap<String, T>) -> NodeDelta<T> {
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
            let old_priv = &old[*key];
            let new_priv = &new[*key];
            if old_priv != new_priv {
                delta.changed.insert((old_priv.clone(), new_priv.clone()));
            }
        }

        delta
    }

    pub fn plan(
        self,
        node: &NodeEnum,
        gen_sql: fn(&NodeEnum, Option<T>, bool) -> anyhow::Result<Vec<String>>,
        gen_sql_for_changed: fn(&NodeEnum, T, T) -> anyhow::Result<Vec<String>>,
    ) -> anyhow::Result<Vec<String>> {
        let mut migrations = Vec::new();
        for removed in self.removed {
            let sqls = gen_sql(node, Some(removed), false)?;
            migrations.extend(sqls);
        }

        for added in self.added {
            let sqls = gen_sql(node, Some(added), true)?;
            migrations.extend(sqls);
        }

        for (v1, v2) in self.changed {
            let sqls = gen_sql_for_changed(node, v1, v2)?;
            migrations.extend(sqls);
        }
        Ok(migrations)
    }
}
