use super::{RelationId, SchemaId};
use std::fmt;

impl SchemaId {
    pub fn new(schema: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            schema: schema.into(),
            name: name.into(),
        }
    }

    pub fn new_with(names: &[&str]) -> Self {
        if names.len() > 2 {
            Self {
                schema: names[0].to_string(),
                name: names[1..].join("."),
            }
        } else {
            Self {
                schema: "public".to_string(),
                name: names[0].to_string(),
            }
        }
    }
}

impl fmt::Display for SchemaId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}", self.schema, self.name)
    }
}

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
