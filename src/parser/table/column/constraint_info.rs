use crate::{
    parser::{utils::node_to_string, ConstraintInfo, Table},
    DeltaItem,
};
use anyhow::Result;
use pg_query::{protobuf::ConstrType, NodeEnum};

impl ConstraintInfo {
    pub(super) fn generate_sql(&self) -> Result<String> {
        let s = match self.node {
            NodeEnum::Constraint(ref constraint)
                if constraint.contype() == ConstrType::ConstrDefault =>
            {
                let expr = constraint.raw_expr.as_deref().unwrap();
                if let Some(s) = node_to_string(expr) {
                    return Ok(format!("DEFAULT {}", s));
                }
                "".to_owned()
            }
            NodeEnum::Constraint(ref constraint)
                if constraint.contype() == ConstrType::ConstrCheck =>
            {
                let expr = constraint.raw_expr.as_deref().unwrap();
                if let Some(s) = node_to_string(expr) {
                    return Ok(format!("CONSTRAINT {} CHECK ({})", self.name, s));
                }
                "".to_owned()
            }
            _ => "".to_owned(),
        };
        Ok(s)
    }
}

impl DeltaItem for ConstraintInfo {
    type SqlNode = Table;
    fn drop(self, item: &Self::SqlNode) -> anyhow::Result<Vec<String>> {
        let sql = format!("ALTER TABLE ONLY {} DROP CONSTRAINT {}", item.id, self.name);

        Ok(vec![sql])
    }

    fn create(self, item: &Self::SqlNode) -> anyhow::Result<Vec<String>> {
        let sql = format!("ALTER TABLE ONLY {} ADD {}", item.id, self.generate_sql()?);
        Ok(vec![sql])
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

#[cfg(test)]
mod tests {
    use crate::parser::Table;
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
        let sql = constraint.generate_sql()?;
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
        let sql = constraint.generate_sql()?;
        assert_eq!(sql, "DEFAULT 'abcd'");
        Ok(())
    }
}
