use log;
use std::time::Duration;
use sea_orm::{
    Database,
    ConnectOptions, DbErr, DatabaseConnection
};

pub mod entities;
pub mod mutation;

pub async fn database_connect() -> Result<DatabaseConnection, DbErr>{
    let mut opt = ConnectOptions::new(String::from("postgres://postgres:dockerdb@localhost:5432/rust-todo"));
    opt.max_connections(100)
        .min_connections(5)
        .connect_timeout(Duration::from_secs(8))
        .acquire_timeout(Duration::from_secs(8))
        .idle_timeout(Duration::from_secs(8))
        .max_lifetime(Duration::from_secs(8))
        .sqlx_logging(true)
        .sqlx_logging_level(log::LevelFilter::Info);
    
    let db = Database::connect(opt).await?;
    Ok(db)
}
