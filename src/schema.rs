use crate::{DatabaseSchema, Differ, MigrationPlanner, NodeDiff, NodeItem};
use anyhow::Result;
use std::{
    collections::{BTreeMap, BTreeSet, HashSet},
    hash::Hash,
    str::FromStr,
};

trait SchemaPlan {
    fn diff_altered(&self, remote: &Self, verbose: bool) -> Result<Vec<String>>;
    fn diff_added(&self, verbose: bool) -> Result<Vec<String>>;
    fn diff_removed(&self, verbose: bool) -> Result<Vec<String>>;
}

impl DatabaseSchema {
    pub fn update_schema_names(&mut self) {
        let mut names = BTreeSet::new();
        names.extend(self.extensions.keys().cloned());
        names.extend(self.composite_types.keys().cloned());
        names.extend(self.enum_types.keys().cloned());
        names.extend(self.sequences.keys().cloned());
        names.extend(self.tables.keys().cloned());
        names.extend(self.views.keys().cloned());
        names.extend(self.mviews.keys().cloned());
        names.extend(self.functions.keys().cloned());
        self.schemas = names;
    }

    pub fn sql(&self, include_schema: bool) -> String {
        let mut sql = String::new();
        if include_schema {
            for schema in &self.schemas {
                sql.push_str(&format!("CREATE SCHEMA IF NOT EXISTS {};\n", schema));
            }
        }

        format!("{}{}", sql, self)
    }

    pub fn plan(&self, other: &Self, verbose: bool) -> anyhow::Result<Vec<String>> {
        let mut migrations: Vec<String> = Vec::new();

        // add schema names
        migrations.extend(schema_name_added(&self.schemas, &other.schemas)?);

        // diff on composite types
        migrations.extend(schema_diff(
            &self.composite_types,
            &other.composite_types,
            verbose,
        )?);
        migrations.extend(schema_diff(&self.enum_types, &other.enum_types, verbose)?);
        // diff on sequences
        migrations.extend(schema_diff(&self.sequences, &other.sequences, verbose)?);
        // diff on tables
        migrations.extend(schema_diff(&self.tables, &other.tables, verbose)?);

        // diff on table related stuff
        migrations.extend(schema_diff(
            &self.table_sequences,
            &other.table_sequences,
            verbose,
        )?);
        migrations.extend(schema_diff(
            &self.table_constraints,
            &other.table_constraints,
            verbose,
        )?);
        migrations.extend(schema_diff(
            &self.table_indexes,
            &other.table_indexes,
            verbose,
        )?);
        migrations.extend(schema_diff(
            &self.table_policies,
            &other.table_policies,
            verbose,
        )?);

        // diff on rls
        migrations.extend(schema_diff(&self.table_rls, &other.table_rls, verbose)?);
        // diff on table owners
        migrations.extend(schema_diff(
            &self.table_owners,
            &other.table_owners,
            verbose,
        )?);

        // diff on views
        migrations.extend(schema_diff(&self.views, &other.views, verbose)?);
        // diff on materialized views
        migrations.extend(schema_diff(&self.mviews, &other.mviews, verbose)?);
        // diff on functions
        migrations.extend(schema_diff(&self.functions, &other.functions, verbose)?);

        // diff on triggers
        migrations.extend(schema_diff(
            &self.table_triggers,
            &other.table_triggers,
            verbose,
        )?);

        // diff on privileges
        migrations.extend(schema_diff(&self.privileges, &other.privileges, verbose)?);

        // finally, drop the schema names
        migrations.extend(schema_name_removed(&self.schemas, &other.schemas)?);

        Ok(migrations)
    }
}

impl<T> SchemaPlan for T
where
    T: NodeItem + Clone + FromStr<Err = anyhow::Error> + PartialEq + Eq + 'static,
    NodeDiff<T>: MigrationPlanner<Migration = String>,
{
    fn diff_altered(&self, remote: &Self, verbose: bool) -> Result<Vec<String>> {
        let diff = remote.diff(self)?;
        if let Some(diff) = diff {
            if verbose && atty::is(atty::Stream::Stdout) {
                println!(
                    "{} {} is changed:\n\n{}",
                    self.type_name(),
                    self.id(),
                    diff.diff
                );
            }
            diff.plan()
        } else {
            Ok(Vec::new())
        }
    }

    fn diff_added(&self, verbose: bool) -> Result<Vec<String>> {
        let diff = NodeDiff::with_new(self.clone());
        if verbose && atty::is(atty::Stream::Stdout) {
            println!(
                "{} {} is added:\n\n{}",
                self.type_name(),
                self.id(),
                diff.diff,
            );
        }
        diff.plan()
    }

    fn diff_removed(&self, verbose: bool) -> Result<Vec<String>> {
        let diff = NodeDiff::with_old(self.clone());
        if verbose && atty::is(atty::Stream::Stdout) {
            println!(
                "{} {} is removed:\n\n{}",
                self.type_name(),
                self.id(),
                diff.diff,
            );
        }
        diff.plan()
    }
}

impl<T> SchemaPlan for BTreeMap<String, T>
where
    T: NodeItem + Clone + FromStr<Err = anyhow::Error> + PartialEq + Eq + 'static,
    NodeDiff<T>: MigrationPlanner<Migration = String>,
{
    fn diff_altered(&self, remote: &Self, verbose: bool) -> Result<Vec<String>> {
        let mut migrations: Vec<String> = Vec::new();
        let keys: HashSet<_> = self.keys().collect();
        let other_keys: HashSet<_> = remote.keys().collect();
        let added = keys.difference(&other_keys);
        for key in added {
            let v = self.get(*key).unwrap().clone();
            let (id, t) = (v.id(), v.type_name());
            let diff = NodeDiff::with_new(v);
            if verbose && atty::is(atty::Stream::Stdout) {
                println!("{} {} is added:\n\n{}", t, id, diff.diff);
            }
            migrations.extend(diff.plan()?);
        }
        let removed = other_keys.difference(&keys);
        for key in removed {
            let v = remote.get(*key).unwrap().clone();
            let (id, t) = (v.id(), v.type_name());
            let diff = NodeDiff::with_old(v);
            if verbose && atty::is(atty::Stream::Stdout) {
                println!("{} {} is removed:\n\n{}", t, id, diff.diff);
            }
            migrations.extend(diff.plan()?);
        }
        let intersection = keys.intersection(&other_keys);
        for key in intersection {
            let local: T = self.get(*key).unwrap().to_string().parse()?;
            let remote: T = remote.get(*key).unwrap().to_string().parse()?;
            migrations.extend(local.diff_altered(&remote, verbose)?);
        }

        Ok(migrations)
    }

    fn diff_added(&self, verbose: bool) -> Result<Vec<String>> {
        let mut migrations: Vec<String> = Vec::new();
        for item in self.values() {
            migrations.extend(item.diff_added(verbose)?);
        }

        Ok(migrations)
    }

    fn diff_removed(&self, verbose: bool) -> Result<Vec<String>> {
        let mut migrations: Vec<String> = Vec::new();
        for item in self.values() {
            migrations.extend(item.diff_removed(verbose)?);
        }
        Ok(migrations)
    }
}

impl<T> SchemaPlan for BTreeSet<T>
where
    T: NodeItem + Clone + FromStr<Err = anyhow::Error> + PartialEq + Eq + Ord + Hash + 'static,
    NodeDiff<T>: MigrationPlanner<Migration = String>,
{
    fn diff_altered(&self, remote: &Self, verbose: bool) -> Result<Vec<String>> {
        let mut migrations: Vec<String> = Vec::new();
        let added = self.difference(remote);
        for v in added {
            let (id, t) = (v.id(), v.type_name());
            let diff = NodeDiff::with_new(v.clone());
            if verbose && atty::is(atty::Stream::Stdout) {
                println!("{} {} is added:\n\n{}", t, id, diff.diff);
            }
            migrations.extend(diff.plan()?);
        }
        let removed = remote.difference(self);
        for v in removed {
            let (id, t) = (v.id(), v.type_name());
            let diff = NodeDiff::with_old(v.clone());
            if verbose && atty::is(atty::Stream::Stdout) {
                println!("{} {} is removed:\n\n{}", t, id, diff.diff);
            }
            migrations.extend(diff.plan()?);
        }

        Ok(migrations)
    }

    fn diff_added(&self, verbose: bool) -> Result<Vec<String>> {
        let mut migrations: Vec<String> = Vec::new();
        for item in self {
            migrations.extend(item.diff_added(verbose)?);
        }

        Ok(migrations)
    }

    fn diff_removed(&self, verbose: bool) -> Result<Vec<String>> {
        let mut migrations: Vec<String> = Vec::new();
        for item in self {
            migrations.extend(item.diff_removed(verbose)?);
        }
        Ok(migrations)
    }
}

fn schema_name_added(local: &BTreeSet<String>, remote: &BTreeSet<String>) -> Result<Vec<String>> {
    let mut migrations: Vec<String> = Vec::new();

    let added = local.difference(remote);
    for key in added {
        migrations.push(format!("CREATE SCHEMA IF NOT EXISTS {}", key));
    }

    Ok(migrations)
}

fn schema_name_removed(local: &BTreeSet<String>, remote: &BTreeSet<String>) -> Result<Vec<String>> {
    let mut migrations: Vec<String> = Vec::new();

    let removed = remote.difference(local);
    for key in removed {
        migrations.push(format!("DROP SCHEMA {}", key));
    }

    Ok(migrations)
}

fn schema_diff<K, T>(
    local: &BTreeMap<K, T>,
    remote: &BTreeMap<K, T>,
    verbose: bool,
) -> Result<Vec<String>>
where
    K: Hash + Eq + Ord,
    T: SchemaPlan,
{
    let mut migrations: Vec<String> = Vec::new();
    let keys: HashSet<_> = local.keys().collect();
    let other_keys: HashSet<_> = remote.keys().collect();

    // process intersection
    let intersection = keys.intersection(&other_keys);
    for key in intersection {
        let local = local.get(*key).unwrap();
        let remote = remote.get(*key).unwrap();
        migrations.extend(local.diff_altered(remote, verbose)?);
    }

    // process added
    let added = keys.difference(&other_keys);
    for key in added {
        let local = local.get(*key).unwrap();
        migrations.extend(local.diff_added(verbose)?);
    }

    // process removed
    let removed = other_keys.difference(&keys);
    for key in removed {
        let remote = remote.get(*key).unwrap();
        migrations.extend(remote.diff_removed(verbose)?);
    }
    Ok(migrations)
}

#[cfg(test)]
mod tests {
    use crate::{SchemaLoader, SqlLoader};

    use super::*;

    #[tokio::test]
    async fn database_schema_plan_should_work() -> Result<()> {
        let loader = SqlLoader::new(
            r#"
            CREATE TYPE public.test_type AS (id uuid, name text);
            CREATE TABLE public.test_table (id uuid, name text);
            CREATE VIEW public.test_view AS SELECT * FROM public.test_table;
            CREATE FUNCTION public.test_function(a text) RETURNS text AS $$ SELECT 'test', a $$ LANGUAGE SQL;
            "#,
        );
        let remote = loader.load().await?;
        let loader = SqlLoader::new(
            r#"
            CREATE TYPE public.test_type AS (id uuid, name text);
            CREATE TABLE public.test_table (id uuid, name text, created_at timestamptz);
            CREATE VIEW public.test_view AS SELECT * FROM public.test_table where created_at > now();
            CREATE FUNCTION public.test_function(a text) RETURNS text AS $$ SELECT a, 'test1' $$ LANGUAGE SQL;
            "#,
        );
        let local = loader.load().await?;
        let migrations = local.plan(&remote, false).unwrap();
        assert_eq!(migrations.len(), 4);
        assert_eq!(
            migrations[0],
            "ALTER TABLE public.test_table ADD COLUMN created_at timestamptz"
        );
        assert_eq!(migrations[1], "DROP VIEW public.test_view");
        assert_eq!(
            migrations[2],
            "CREATE VIEW public.test_view AS SELECT * FROM public.test_table WHERE created_at > now()"
        );
        assert_eq!(
            migrations[3],
            "CREATE OR REPLACE FUNCTION public.test_function(a text) RETURNS text AS $$ SELECT a, 'test1' $$ LANGUAGE sql"
        );

        Ok(())
    }
}
