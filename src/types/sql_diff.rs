use crate::{utils::create_diff, DiffItem, MigrationPlanner, SqlDiff, SqlDiffer};

impl<T> SqlDiffer for T
where
    T: PartialEq + Clone + DiffItem,
    SqlDiff<T>: MigrationPlanner,
{
    type Delta = SqlDiff<T>;
    fn diff(&self, remote: &Self) -> anyhow::Result<Option<Self::Delta>> {
        let local_id = self.id();
        let remote_id = remote.id();
        if local_id != remote_id {
            anyhow::bail!("can't diff {} and {}", local_id, remote_id);
        }

        if self != remote {
            let diff = create_diff(self.node(), remote.node())?;
            Ok(Some(SqlDiff {
                old: Some(self.clone()),
                new: Some(remote.clone()),
                diff,
            }))
        } else {
            Ok(None)
        }
    }
}
