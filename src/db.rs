pub mod movie_servings;
pub mod prelude;

pub use migration;
use std::fs::create_dir_all;

use rocket::figment::Figment;
use rocket_db_pools::Database;
use sea_orm::*;

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
        let settings_dir = home::home_dir().unwrap().join(".mpvserve");
        match create_dir_all(&settings_dir) {
            Err(e) => {
                return Err(Self::Error::Custom(format!(
                    "failed to create settings directory at {:?}: {:?}",
                    settings_dir, e
                )))
            }
            _ => {}
        }

        let db_file = settings_dir.join("data.db");
        let db_url = format!("sqlite://{}?mode=rwc", &db_file.to_str().unwrap());

        let conn = sea_orm::Database::connect(ConnectOptions::new(db_url))
            .await
            .unwrap();
        Ok(DbPool { conn })
    }

    async fn get(&self) -> Result<Self::Connection, Self::Error> {
        Ok(self.conn.clone())
    }
}
