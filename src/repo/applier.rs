use crate::{utils::load_config, DatabaseSchema, RemoteRepo, SchemaLoader, SqlSaver};
use anyhow::Result;
use sqlx::{Connection, Executor, PgConnection};

impl RemoteRepo {
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

    pub async fn fetch(&self) -> Result<DatabaseSchema> {
        let schema = self.load().await?;
        let config = load_config().await?;
        schema.save(&config.output).await?;
        Ok(schema)
    }
}
