use super::{SchemaId, Sequence};
use crate::NodeItem;
use pg_query::{protobuf::CreateSeqStmt, NodeEnum, NodeRef};

impl NodeItem for Sequence {
    type Inner = CreateSeqStmt;

    fn id(&self) -> String {
        self.id.to_string()
    }

    fn type_name(&self) -> &'static str {
        "sequence"
    }

    fn node(&self) -> &NodeEnum {
        &self.node
    }

    fn inner(&self) -> anyhow::Result<&Self::Inner> {
        match &self.node {
            NodeEnum::CreateSeqStmt(stmt) => Ok(stmt),
            _ => anyhow::bail!("not a create sequence statement"),
        }
    }

    fn revert(&self) -> anyhow::Result<NodeEnum> {
        let sql = format!("DROP SEQUENCE {}", self.id);
        let parsed = pg_query::parse(&sql)?;
        let node = parsed.protobuf.nodes()[0].0;
        match node {
            NodeRef::DropStmt(stmt) => Ok(NodeEnum::DropStmt(stmt.clone())),
            _ => anyhow::bail!("not a drop statement"),
        }
    }
}

impl TryFrom<&CreateSeqStmt> for Sequence {
    type Error = anyhow::Error;
    fn try_from(stmt: &CreateSeqStmt) -> Result<Self, Self::Error> {
        let id = SchemaId::from(stmt.sequence.as_ref());
        let node = NodeEnum::CreateSeqStmt(stmt.clone());
        Ok(Self { id, node })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Differ, MigrationPlanner};

    #[test]
    fn sequence_should_parse() {
        let sql = "CREATE SEQUENCE public.todos_id_seq
        START WITH 1
        INCREMENT BY 1
        NO MINVALUE
        NO MAXVALUE
        CACHE 1;";
        let seq: Sequence = sql.parse().unwrap();
        assert_eq!(seq.id.to_string(), "public.todos_id_seq");
    }

    #[test]
    fn test_sequence_migration() {
        let sql1 = "CREATE SEQUENCE public.todos_id_seq START WITH 1 INCREMENT BY 1 NO MINVALUE NO MAXVALUE CACHE 1";
        let sql2 = "CREATE SEQUENCE public.todos_id_seq START 1 INCREMENT 2 NO MINVALUE NO MAXVALUE CACHE 1";
        let old: Sequence = sql1.parse().unwrap();
        let new: Sequence = sql2.parse().unwrap();
        let diff = old.diff(&new).unwrap().unwrap();
        let migrations = diff.plan().unwrap();
        assert_eq!(migrations.len(), 2);
        assert_eq!(migrations[0], "DROP SEQUENCE public.todos_id_seq");
        assert_eq!(migrations[1], sql2);
    }
}
