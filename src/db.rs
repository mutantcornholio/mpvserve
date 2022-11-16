pub mod movie_servings;
pub mod prelude;
pub use migration;

use rocket::figment::Figment;
use rocket_db_pools::{Database};
use sea_orm::*;
use home;

#[derive(Database)]
#[database("sea_orm")]
pub struct Db(DbPool);

pub struct DbPool {
    pub conn: DatabaseConnection,
}

#[async_trait]
impl rocket_db_pools::Pool for DbPool {
    type Connection = DatabaseConnection;
    type Error = DbErr;

    async fn init(_figment: &Figment) -> Result<Self, Self::Error> {
        let home_dir = home::home_dir().unwrap().join(".mpvserve").join("data.db");
        let url = home_dir.to_str().unwrap();
        let url = format!("sqlite://{}?mode=rwc", url);

        let conn = sea_orm::Database::connect(ConnectOptions::new(url))
            .await
            .unwrap();
        Ok(DbPool { conn })
    }

    async fn get(&self) -> Result<Self::Connection, Self::Error> {
        Ok(self.conn.clone())
    }
}
