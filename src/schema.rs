use crate::{
    parser::{CompositeType, Function, Table, View},
    schema_diff, DatabaseSchema, Differ, MigrationPlanner, NodeDiff, NodeItem,
};
use std::collections::HashSet;

impl DatabaseSchema {
    pub fn plan(&self, other: &Self, verbose: bool) -> anyhow::Result<Vec<String>> {
        let mut migrations: Vec<String> = Vec::new();

        // diff on types
        schema_diff!(
            &self.composite_types,
            &other.composite_types,
            migrations,
            CompositeType,
            verbose
        );
        // diff on tables
        schema_diff!(&self.tables, &other.tables, migrations, Table, verbose);
        // diff on views
        schema_diff!(&self.views, &other.views, migrations, View, verbose);
        // diff on functions
        schema_diff!(
            &self.functions,
            &other.functions,
            migrations,
            Function,
            verbose
        );

        Ok(migrations)
    }
}
