use crate::{
    parser::Table, schema_diff, DatabaseSchema, Differ, MigrationPlanner, NodeDiff, NodeItem,
};
use std::collections::HashSet;

impl DatabaseSchema {
    pub fn plan(&self, other: &Self) -> anyhow::Result<Vec<String>> {
        let mut migrations: Vec<String> = Vec::new();

        // diff on types
        schema_diff!(
            &self.composite_types,
            &other.composite_types,
            migrations,
            Table
        );
        // diff on tables
        schema_diff!(&self.tables, &other.tables, migrations, Table);
        // diff on views
        schema_diff!(&self.views, &other.views, migrations, Table);
        // diff on functions
        schema_diff!(&self.functions, &other.functions, migrations, Table);

        Ok(migrations)
    }
}
