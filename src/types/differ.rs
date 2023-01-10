use crate::{
    utils::{create_diff, create_diff_added, create_diff_removed},
    Differ, MigrationPlanner, NodeDiff, NodeItem,
};

impl<T> Differ for T
where
    T: PartialEq + Clone + NodeItem + ToString,
    NodeDiff<T>: MigrationPlanner,
{
    type Diff = NodeDiff<T>;
    fn diff(&self, new: &Self) -> anyhow::Result<Option<Self::Diff>> {
        let old_id = self.id();
        let new_id = new.id();
        if old_id != new_id {
            anyhow::bail!("can't diff {} and {}", old_id, new_id);
        }

        let self_str = self.to_string();
        let new_str = new.to_string();
        if self_str != new_str {
            let diff = create_diff(self, new)?;
            Ok(Some(NodeDiff {
                old: Some(self.clone()),
                new: Some(new.clone()),
                diff,
            }))
        } else {
            Ok(None)
        }
    }
}

impl<T> NodeDiff<T>
where
    T: NodeItem,
{
    pub fn with_old(old: T) -> Self {
        let diff = create_diff_removed(&old).unwrap();
        Self {
            old: Some(old),
            new: None,
            diff,
        }
    }

    pub fn with_new(new: T) -> Self {
        let diff = create_diff_added(&new).unwrap();
        Self {
            old: None,
            new: Some(new),
            diff,
        }
    }
}
