use crate::parser::SchemaId;
use pg_query::protobuf::RangeVar;
use std::{fmt, str::FromStr};

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

impl From<&RangeVar> for SchemaId {
    fn from(v: &RangeVar) -> Self {
        let schema_name = if v.schemaname.is_empty() {
            "public"
        } else {
            v.schemaname.as_str()
        };
        Self::new(schema_name, &v.relname)
    }
}

impl From<Option<&RangeVar>> for SchemaId {
    fn from(v: Option<&RangeVar>) -> Self {
        assert!(v.is_some());
        v.unwrap().into()
    }
}

impl FromStr for SchemaId {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<_> = s.split('.').collect();
        Ok(Self::new_with(&parts))
    }
}

impl fmt::Display for SchemaId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}", self.schema, self.name)
    }
}
