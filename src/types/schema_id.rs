use pg_query::protobuf::RangeVar;
use std::fmt;

use crate::parser::SchemaId;

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

impl From<Option<&RangeVar>> for SchemaId {
    fn from(v: Option<&RangeVar>) -> Self {
        assert!(v.is_some());
        let v = v.unwrap();

        let schema = if v.schemaname.is_empty() {
            "public"
        } else {
            v.schemaname.as_str()
        };

        Self::new(schema, &v.relname)
    }
}

impl fmt::Display for SchemaId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}", self.schema, self.name)
    }
}
