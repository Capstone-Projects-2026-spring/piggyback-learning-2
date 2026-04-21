use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};
use std::fs;
use std::sync::OnceLock;

use super::schema::CREATE_TABLES;

static DB: OnceLock<SqlitePool> = OnceLock::new();

pub fn get_db() -> &'static SqlitePool {
    DB.get()
        .expect("[db] not initialised — call init_db() first")
}

pub struct FirstRunInfo {
    pub is_first_run: bool,
    pub db_path: std::path::PathBuf,
}

pub async fn init_db() -> Result<FirstRunInfo, String> {
    let data_dir = dirs::data_local_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("piggyback");

    fs::create_dir_all(&data_dir).map_err(|e| format!("[db] failed to create data dir: {e}"))?;

    let db_path = data_dir.join("piggyback.db");
    let is_first_run = !db_path.exists();

    eprintln!("[db] path={} first_run={is_first_run}", db_path.display());

    let url = format!("sqlite://{}?mode=rwc", db_path.display());
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&url)
        .await
        .map_err(|e| format!("[db] connect failed: {e}"))?;

    // Run schema
    sqlx::raw_sql(CREATE_TABLES)
        .execute(&pool)
        .await
        .map_err(|e| format!("[db] schema error: {e}"))?;

    // Stamp first-run metadata
    if is_first_run {
        sqlx::query(
            "INSERT OR IGNORE INTO app_meta (key, value) VALUES ('first_run_at', datetime('now'))",
        )
        .execute(&pool)
        .await
        .map_err(|e| format!("[db] meta insert: {e}"))?;
        eprintln!("[db] first run — tables created and stamped");
    } else {
        eprintln!("[db] existing db loaded");
    }

    DB.set(pool)
        .map_err(|_| "[db] already initialised".to_string())?;

    Ok(FirstRunInfo {
        is_first_run,
        db_path,
    })
}
