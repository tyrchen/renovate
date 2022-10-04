use std::collections::BTreeSet;

use crate::NodeDelta;

mod differ;
mod node_delta;
mod relation_id;
mod schema_id;

impl<T> Default for NodeDelta<T> {
    fn default() -> Self {
        Self {
            added: BTreeSet::new(),
            removed: BTreeSet::new(),
            changed: BTreeSet::new(),
        }
    }
}
