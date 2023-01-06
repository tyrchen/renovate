use crate::{utils::load_config, RemoteRepo, SchemaLoader, SqlSaver};
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

        let schema = self.load().await?;

        let config = load_config().await?;
        schema.save(&config.output).await?;
        Ok(())
    }
}
