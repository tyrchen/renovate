use crate::NodeDelta;
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
}
