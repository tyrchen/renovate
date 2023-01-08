use std::thread;

use crate::{utils::load_config, DatabaseSchema, RemoteRepo, SchemaLoader, SqlSaver};
use anyhow::Result;
use sqlx::{Connection, Executor, PgConnection};
use tokio::runtime::Runtime;
use url::Url;
use uuid::Uuid;

impl RemoteRepo {
    pub async fn normalize(&self, sql: &str) -> Result<DatabaseSchema> {
        let tdb = TmpDb::new(self.server_url()?, sql).await?;
        let repo = RemoteRepo::new(tdb.url());
        repo.load().await
    }

    /// Apply the migration plan to the remote database server.
    pub async fn apply(&self, plan: Vec<String>) -> Result<()> {
        let mut conn = PgConnection::connect(&self.url).await?;
        let mut tx = conn.begin().await?;

        for sql in plan {
            tx.execute(sql.as_str()).await?;
        }
        tx.commit().await?;

        self.fetch().await?;
        Ok(())
    }

    /// Fetch the most recent schema from the remote database server.
    pub async fn fetch(&self) -> Result<DatabaseSchema> {
        let schema = self.load().await?;
        let config = load_config().await?;
        schema.save(&config.output).await?;
        Ok(schema)
    }

    fn server_url(&self) -> Result<String> {
        let mut url = Url::parse(&self.url)?;
        url.set_path("");
        Ok(url.to_string())
    }
}

#[derive(Debug)]
pub struct TmpDb {
    pub server_url: String,
    pub dbname: String,
}

impl TmpDb {
    pub async fn new(server_url: String, sql: &str) -> Result<Self> {
        let dbname = format!("tmpdb_{}", Uuid::new_v4());
        let dbname_cloned = dbname.clone();
        let tdb = Self { server_url, dbname };

        let server_url = tdb.server_url();
        let url = tdb.url();

        // create database dbname
        // use server url to create database
        let mut conn = PgConnection::connect(&server_url).await?;
        conn.execute(format!(r#"CREATE DATABASE "{}""#, dbname_cloned).as_str())
            .await?;

        // now connect to test database for migration
        let mut conn = PgConnection::connect(&url).await?;
        let mut tx = conn.begin().await?;
        // for stmt in sql.split(';') {
        //     if stmt.trim().is_empty() || stmt.starts_with("--") {
        //         continue;
        //     }
        //     tx.execute(stmt).await?;
        // }
        tx.execute(sql).await?;
        tx.commit().await?;

        Ok(tdb)
    }

    pub fn server_url(&self) -> String {
        self.server_url.clone()
    }

    pub fn url(&self) -> String {
        format!("{}/{}", self.server_url, self.dbname)
    }
}

impl Drop for TmpDb {
    fn drop(&mut self) {
        let server_url = self.server_url();
        let dbname = self.dbname.clone();
        thread::spawn(move || {
            let rt = Runtime::new().unwrap();
            rt.block_on(async move {
                    let mut conn = PgConnection::connect(&server_url).await.unwrap();
                    // terminate existing connections
                    sqlx::query(&format!(r#"SELECT pg_terminate_backend(pid) FROM pg_stat_activity WHERE pid <> pg_backend_pid() AND datname = '{}'"#, dbname))
                    .execute( &mut conn)
                    .await
                    .expect("Terminate all other connections");
                    conn.execute(format!(r#"DROP DATABASE "{}""#, dbname).as_str())
                        .await
                        .expect("Error while querying the drop database");
                });
            })
            .join()
            .expect("failed to drop database");
    }
}
