use crate::models::Inbox;
use anyhow::{anyhow, Result};
use sqlx::{sqlite::SqliteConnectOptions, ConnectOptions, SqlitePool};
use std::str::FromStr;

pub async fn init_db(database_url: &str) -> Result<SqlitePool> {
    let mut options = SqliteConnectOptions::from_str(database_url)
        .map_err(|e| anyhow!("Invalid database URL: {}", e))?
        .create_if_missing(true);

    // Disable statement logging spam
    options = options.log_statements(tracing::log::LevelFilter::Debug);

    let pool = SqlitePool::connect_with(options).await?;

    // Enable WAL mode and foreign keys
    sqlx::query("PRAGMA journal_mode = WAL;").execute(&pool).await?;
    sqlx::query("PRAGMA foreign_keys = ON;").execute(&pool).await?;

    // Run schema migrations embedded
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .map_err(|e| anyhow!("Failed to run database migrations: {}", e))?;

    Ok(pool)
}

pub async fn get_or_create_inbox(pool: &SqlitePool, name: &str) -> Result<Inbox> {
    let clean_name = name.trim().to_lowercase();
    
    // Check if exists
    if let Some(inbox) = sqlx::query_as::<_, Inbox>("SELECT * FROM inboxes WHERE name = ?")
        .bind(&clean_name)
        .fetch_optional(pool)
        .await?
    {
        return Ok(inbox);
    }

    // Create new
    sqlx::query(
        "INSERT INTO inboxes (name, is_locked, created_at, last_user_activity_at) VALUES (?, 0, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)"
    )
    .bind(&clean_name)
    .execute(pool)
    .await?;

    let inbox = sqlx::query_as::<_, Inbox>("SELECT * FROM inboxes WHERE name = ?")
        .bind(&clean_name)
        .fetch_one(pool)
        .await?;

    Ok(inbox)
}

pub async fn touch_user_activity(pool: &SqlitePool, name: &str) -> Result<()> {
    let clean_name = name.trim().to_lowercase();
    sqlx::query("UPDATE inboxes SET last_user_activity_at = CURRENT_TIMESTAMP WHERE name = ?")
        .bind(&clean_name)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn lock_inbox(pool: &SqlitePool, name: &str, password_hash: &str) -> Result<()> {
    let clean_name = name.trim().to_lowercase();
    
    // 1. Purge existing public messages to prevent snooping verification codes
    sqlx::query("DELETE FROM messages WHERE inbox_name = ?")
        .bind(&clean_name)
        .execute(pool)
        .await?;

    // 2. Set locked state and password hash
    sqlx::query(
        "UPDATE inboxes SET is_locked = 1, password_hash = ?, locked_at = CURRENT_TIMESTAMP, last_user_activity_at = CURRENT_TIMESTAMP WHERE name = ?"
    )
    .bind(password_hash)
    .bind(&clean_name)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn remove_lock(pool: &SqlitePool, name: &str) -> Result<()> {
    let clean_name = name.trim().to_lowercase();
    sqlx::query(
        "UPDATE inboxes SET is_locked = 0, password_hash = NULL, locked_at = NULL, last_user_activity_at = CURRENT_TIMESTAMP WHERE name = ?"
    )
    .bind(&clean_name)
    .execute(pool)
    .await?;
    Ok(())
}
