use super::schema::CREATE_TABLES;
use crate::utils::crypto;

use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};
use std::{fs, sync::OnceLock};

static DB: OnceLock<SqlitePool> = OnceLock::new();

pub fn get_db() -> &'static SqlitePool {
    DB.get()
        .expect("[db] not initialised — call init_db() at startup")
}

pub async fn init_db() -> Result<std::path::PathBuf, String> {
    let data_dir = dirs::data_local_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("piggyback");

    fs::create_dir_all(&data_dir).map_err(|e| format!("[db] create data dir failed: {e}"))?;

    let db_path = data_dir.join("piggyback.db");
    let is_first_run = !db_path.exists();

    eprintln!("[db] path={} first_run={is_first_run}", db_path.display());

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&format!("sqlite://{}?mode=rwc", db_path.display()))
        .await
        .map_err(|e| format!("[db] connect failed: {e}"))?;

    sqlx::raw_sql(CREATE_TABLES)
        .execute(&pool)
        .await
        .map_err(|e| format!("[db] schema failed: {e}"))?;

    if is_first_run {
        sqlx::query(
            "INSERT OR IGNORE INTO app_meta (key, value) VALUES ('first_run_at', datetime('now'))",
        )
        .execute(&pool)
        .await
        .map_err(|e| format!("[db] first-run stamp failed: {e}"))?;
        eprintln!("[db] first run — schema created");
    } else {
        eprintln!("[db] existing db loaded");
    }

    DB.set(pool)
        .map_err(|_| "[db] already initialised".to_string())?;

    // Crypto key is independent of DB but initialised at the same time
    // since both are needed before any handler runs.
    crypto::init_voice_key().map_err(|e| format!("[db] crypto init failed: {e}"))?;

    Ok(db_path)
}

pub async fn has_parent_account() -> bool {
    match sqlx::query_as::<_, (i64,)>("SELECT COUNT(*) FROM users WHERE role = 'parent'")
        .fetch_one(get_db())
        .await
    {
        Ok((count,)) => count > 0,
        Err(e) => {
            eprintln!("[db] has_parent_account failed: {e}");
            false
        }
    }
}
