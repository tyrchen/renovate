use crate::{
    parser::{utils::node_to_string, ConstraintInfo, Table},
    DeltaItem,
};
use pg_query::{protobuf::ConstrType, NodeEnum};
use std::fmt;

impl ConstraintInfo {}

impl DeltaItem for ConstraintInfo {
    type SqlNode = Table;
    fn drop(self, item: &Self::SqlNode) -> anyhow::Result<Vec<String>> {
        let sql = format!("ALTER TABLE ONLY {} DROP CONSTRAINT {}", item.id, self.name);

        Ok(vec![sql])
    }

    fn create(self, item: &Self::SqlNode) -> anyhow::Result<Vec<String>> {
        let sql = format!("ALTER TABLE ONLY {} ADD {}", item.id, self);
        Ok(vec![sql])
    }

    fn rename(self, item: &Self::SqlNode, new: Self) -> anyhow::Result<Vec<String>> {
        let sql1 = self.to_string().replace(&self.name, &new.name);
        let sql2 = new.to_string();
        if sql1 == sql2 {
            return Ok(vec![format!(
                "ALTER TABLE ONLY {} RENAME CONSTRAINT {} TO {}",
                item.id, self.name, new.name
            )]);
        }
        Ok(vec![])
    }

    fn alter(self, item: &Self::SqlNode, new: Self) -> anyhow::Result<Vec<String>> {
        let mut migrations = vec![];
        let sql = self.drop(item)?;
        migrations.extend(sql);
        let sql = new.create(item)?;
        migrations.extend(sql);
        Ok(migrations)
    }
}

impl fmt::Display for ConstraintInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self.node {
            NodeEnum::Constraint(ref constraint)
                if constraint.contype() == ConstrType::ConstrDefault =>
            {
                let expr = constraint.raw_expr.as_deref().unwrap();
                format!("DEFAULT {}", node_to_string(expr).unwrap())
            }
            NodeEnum::Constraint(ref constraint)
                if constraint.contype() == ConstrType::ConstrCheck =>
            {
                let expr = constraint.raw_expr.as_deref().unwrap();
                format!(
                    "CONSTRAINT {} CHECK ({})",
                    self.name,
                    node_to_string(expr).unwrap()
                )
            }
            // TODO: support other constraints (primary key / unique will be normalized to a separate SQL).
            NodeEnum::Constraint(ref _constraint) => "".to_owned(),
            ref v => unreachable!(
                "ConstraintInfo::generate_sql: node should only be constraint, got {:?}",
                v
            ),
        };
        write!(f, "{}", s)
    }
}

#[cfg(test)]
mod tests {
    use crate::{parser::Table, Differ, MigrationPlanner};
    use anyhow::Result;

    #[test]
    fn test_constraint_info_for_default_function() -> Result<()> {
        let sql = "CREATE TABLE foo (name text DEFAULT random_name(1))";
        let table: Table = sql.parse()?;
        let constraint = table
            .columns
            .get("name")
            .as_ref()
            .unwrap()
            .default
            .as_ref()
            .unwrap();
        let sql = constraint.to_string();
        assert_eq!(sql, "DEFAULT random_name(1)");
        Ok(())
    }

    #[test]
    fn test_constraint_info_for_default_value() -> Result<()> {
        let sql = "CREATE TABLE foo (name text DEFAULT 'abcd')";
        let table: Table = sql.parse()?;
        let constraint = table
            .columns
            .get("name")
            .as_ref()
            .unwrap()
            .default
            .as_ref()
            .unwrap();
        let sql = constraint.to_string();
        assert_eq!(sql, "DEFAULT 'abcd'");
        Ok(())
    }

    #[test]
    fn table_rename_constraint_should_work() {
        let s1 = "CREATE TABLE foo (name text, constraint c1 CHECK (length(name) > 5))";
        let s2 = "CREATE TABLE foo (name text, constraint c2 CHECK (length(name) > 5))";
        let old: Table = s1.parse().unwrap();
        let new: Table = s2.parse().unwrap();
        let diff = old.diff(&new).unwrap().unwrap();
        let plan = diff.plan().unwrap();
        assert_eq!(plan.len(), 1);
        assert_eq!(
            plan[0],
            "ALTER TABLE ONLY public.foo RENAME CONSTRAINT c1 TO c2"
        );
    }
}
