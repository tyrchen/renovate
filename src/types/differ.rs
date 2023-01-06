use crate::{utils::create_diff, Differ, MigrationPlanner, NodeDiff, NodeItem};

impl<T> Differ for T
where
    T: PartialEq + Clone + NodeItem + ToString,
    NodeDiff<T>: MigrationPlanner,
{
    type Diff = NodeDiff<T>;
    fn diff(&self, remote: &Self) -> anyhow::Result<Option<Self::Diff>> {
        let local_id = self.id();
        let remote_id = remote.id();
        if local_id != remote_id {
            anyhow::bail!("can't diff {} and {}", local_id, remote_id);
        }

        if self != remote {
            let diff = create_diff(self, remote)?;
            Ok(Some(NodeDiff {
                old: Some(self.clone()),
                new: Some(remote.clone()),
                diff,
            }))
        } else {
            Ok(None)
        }
    }
}

impl<T> NodeDiff<T> {
    pub fn with_old(old: T) -> Self {
        Self {
            old: Some(old),
            new: None,
            diff: "".to_owned(),
        }
    }

    pub fn with_new(new: T) -> Self {
        Self {
            old: None,
            new: Some(new),
            diff: "".to_owned(),
        }
    }
}
