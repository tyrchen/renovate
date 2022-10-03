use crate::parser::{RelationId, SchemaId};

impl RelationId {
    pub fn new(
        schema: impl Into<String>,
        relation: impl Into<String>,
        name: impl Into<String>,
    ) -> Self {
        Self {
            schema_id: SchemaId::new(schema, relation),
            name: name.into(),
        }
    }

    pub fn new_with(schema_id: SchemaId, name: impl Into<String>) -> Self {
        Self {
            schema_id,
            name: name.into(),
        }
    }
}
