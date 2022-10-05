use crate::{
    parser::{AlterTable, AlterTableAction, ConstraintInfo, RelationId, SchemaId, TableConstraint},
    NodeItem,
};
use pg_query::{
    protobuf::{AlterTableStmt, ConstrType, Constraint as PgConstraint},
    NodeEnum, NodeRef,
};

impl NodeItem for TableConstraint {
    type Inner = AlterTableStmt;

    fn id(&self) -> String {
        self.id.name.clone()
    }

    fn node(&self) -> &NodeEnum {
        &self.node
    }

    fn inner(&self) -> anyhow::Result<&Self::Inner> {
        match &self.node {
            NodeEnum::AlterTableStmt(stmt) => Ok(stmt),
            _ => anyhow::bail!("not a alter table statement"),
        }
    }

    fn revert(&self) -> anyhow::Result<NodeEnum> {
        let sql = format!(
            "ALTER TABLE ONLY {} DROP CONSTRAINT {}",
            self.id.schema_id, self.id.name
        );
        let parsed = pg_query::parse(&sql)?;
        let node = parsed.protobuf.nodes()[0].0;
        match node {
            NodeRef::AlterTableStmt(stmt) => Ok(NodeEnum::AlterTableStmt(stmt.clone())),
            _ => anyhow::bail!("not a alter table drop constraint statement"),
        }
    }
}

impl TryFrom<AlterTable> for TableConstraint {
    type Error = anyhow::Error;
    fn try_from(AlterTable { id, action, node }: AlterTable) -> Result<Self, Self::Error> {
        match action {
            AlterTableAction::Constraint(info) => Ok(TableConstraint::new(id, *info, node)),
            _ => anyhow::bail!("not an add constraint"),
        }
    }
}

impl TableConstraint {
    fn new(id: SchemaId, info: ConstraintInfo, node: NodeEnum) -> Self {
        let id = RelationId::new_with(id, info.name.clone());
        Self { id, info, node }
    }
}

impl TryFrom<&PgConstraint> for ConstraintInfo {
    type Error = anyhow::Error;
    fn try_from(constraint: &PgConstraint) -> Result<Self, Self::Error> {
        let con_type = ConstrType::from_i32(constraint.contype).unwrap();
        let node = NodeEnum::Constraint(Box::new(constraint.clone()));
        let name = constraint.conname.clone();
        Ok(Self {
            name,
            con_type,
            node,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Differ, MigrationPlanner};

    #[test]
    fn alter_table_constraint_should_parse() {
        let sql = "ALTER TABLE ONLY users ADD CONSTRAINT users_pkey PRIMARY KEY (id)";
        let parsed: TableConstraint = sql.parse().unwrap();
        assert_eq!(parsed.id.name, "users_pkey");
        assert_eq!(parsed.id.schema_id.to_string(), "public.users");
        assert_eq!(parsed.info.name, "users_pkey");
        assert_eq!(parsed.info.con_type, ConstrType::ConstrPrimary);
    }

    #[test]
    fn alter_table_constraint_should_revert() {
        let sql = "ALTER TABLE ONLY users ADD CONSTRAINT users_pkey PRIMARY KEY (id)";
        let parsed: TableConstraint = sql.parse().unwrap();
        let reverted = parsed.revert().unwrap().deparse().unwrap();
        assert_eq!(
            reverted,
            "ALTER TABLE ONLY public.users DROP CONSTRAINT users_pkey"
        );
    }

    #[test]
    fn alter_table_constraint_migration_should_drop_and_create() {
        let sql1 = "ALTER TABLE ONLY users ADD CONSTRAINT users_pkey PRIMARY KEY (id)";
        let sql2 = "ALTER TABLE ONLY users ADD CONSTRAINT users_pkey PRIMARY KEY (id, name)";
        let old: TableConstraint = sql1.parse().unwrap();
        let new: TableConstraint = sql2.parse().unwrap();
        let diff = Differ::diff(&old, &new).unwrap().unwrap();
        let plan = diff.plan().unwrap();
        assert_eq!(plan.len(), 2);
        assert_eq!(
            plan[0],
            "ALTER TABLE ONLY public.users DROP CONSTRAINT users_pkey"
        );
        assert_eq!(
            plan[1],
            "ALTER TABLE ONLY users ADD CONSTRAINT users_pkey PRIMARY KEY (id, name)"
        );
    }
}
