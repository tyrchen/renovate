use std::thread;

use crate::{utils::load_config, DatabaseRepo, DatabaseSchema, SchemaLoader, SqlSaver};
use anyhow::{bail, Result};
use sqlx::{Connection, Executor, PgConnection};
use tokio::runtime::Runtime;
use url::Url;
use uuid::Uuid;

impl DatabaseRepo {
    pub async fn load_sql_string(&self, remote: bool) -> Result<String> {
        let url = if remote { &self.remote_url } else { &self.url };

        let output = async_process::Command::new("pg_dump")
            .arg("-s")
            .arg(url)
            .output()
            .await?;

        if !output.status.success() {
            bail!("{}", String::from_utf8(output.stderr)?);
        }

        let sql = String::from_utf8(output.stdout)?;
        Ok(sql)
    }
    pub async fn normalize(&self, sql: &str) -> Result<DatabaseSchema> {
        let tdb = TmpDb::new(self.server_url()?, sql).await?;
        let repo = DatabaseRepo::new_with(tdb.url());
        repo.load().await
    }

    /// Apply the migration plan to the remote database server.
    pub async fn apply(&self, plan: Vec<String>, remote: bool) -> Result<()> {
        // apply to local database
        self.do_apply(&plan, &self.url).await?;

        // if local is not equal to remote, apply to remote database if remote is true
        if self.url != self.remote_url && remote {
            self.do_apply(&plan, &self.remote_url).await?;
        }
        Ok(())
    }

    /// Fetch the most recent schema from the remote database server.
    pub async fn fetch(&self) -> Result<DatabaseSchema> {
        let schema = self.load().await?;
        let config = load_config().await?;
        schema.save(&config.output).await?;
        Ok(schema)
    }

    /// create & init local database if not exists
    pub async fn init_local_database(&self) -> Result<()> {
        let ret = PgConnection::connect(&self.url).await;
        match ret {
            Ok(_) => Ok(()),
            Err(_) => {
                let server_url = self.server_url()?;
                let sql = if self.url != self.remote_url {
                    self.load_sql_string(true).await.ok()
                } else {
                    None
                };
                init_database(&server_url, &self.db_name()?, &sql.unwrap_or_default()).await?;

                Ok(())
            }
        }
    }

    /// drop database
    pub async fn drop_database(&self) -> Result<()> {
        drop_database(&self.server_url()?, &self.db_name()?).await
    }

    async fn do_apply(&self, plan: &[String], url: &str) -> Result<()> {
        let mut conn = PgConnection::connect(url).await?;
        let mut tx = conn.begin().await?;

        for sql in plan {
            tx.execute(sql.as_str()).await?;
        }
        tx.commit().await?;

        self.fetch().await?;
        Ok(())
    }

    fn server_url(&self) -> Result<String> {
        let mut url = Url::parse(&self.url)?;
        url.set_path("");
        Ok(url.to_string())
    }

    fn db_name(&self) -> Result<String> {
        let url = Url::parse(&self.url)?;
        let path = url.path();
        let db_name = path.trim_start_matches('/');
        Ok(db_name.to_string())
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
        init_database(&server_url, &dbname, sql).await?;
        Ok(Self { server_url, dbname })
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
                drop_database(&server_url, &dbname).await.unwrap();
            });
        })
        .join()
        .expect("failed to drop database");
    }
}

async fn init_database(server_url: &str, dbname: &str, sql: &str) -> Result<()> {
    // create database dbname
    // use server url to create database
    let mut conn = PgConnection::connect(server_url).await?;
    conn.execute(format!(r#"CREATE DATABASE "{}""#, dbname).as_str())
        .await?;

    // now connect to test database for migration
    let url = format!("{}/{}", server_url, dbname);
    let mut conn = PgConnection::connect(&url).await?;
    let mut tx = conn.begin().await?;
    tx.execute(sql).await?;
    tx.commit().await?;
    Ok(())
}

async fn drop_database(server_url: &str, dbname: &str) -> Result<()> {
    let mut conn = PgConnection::connect(server_url).await?;
    // terminate existing connections
    sqlx::query(&format!(r#"SELECT pg_terminate_backend(pid) FROM pg_stat_activity WHERE pid <> pg_backend_pid() AND datname = '{}'"#, dbname))
                    .execute( &mut conn)
                    .await
                    .expect("Terminate all other connections");
    conn.execute(format!(r#"DROP DATABASE "{}""#, dbname).as_str())
        .await?;

    Ok(())
}
