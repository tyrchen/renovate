use crate::parser::{utils::node_to_string, ConstraintInfo};
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
            _ => "".to_owned(),
        };
        Ok(s)
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
