pub mod models;
pub mod schema;

use log::info;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use std::fs;
use std::path::PathBuf;

pub type DbPool = Pool<SqliteConnectionManager>;

/// Get the workspace directory path: ~/TopicExplorer/.workspace/
pub fn get_workspace_dir() -> PathBuf {
    let home = dirs::home_dir().expect("Could not determine home directory");
    home.join("TopicExplorer").join(".workspace")
}

/// Get the database file path
pub fn get_db_path() -> PathBuf {
    get_workspace_dir().join("topic_explorer.db")
}

/// Initialize the database: create workspace directory, open/create SQLite DB, run migrations
pub fn init_db() -> Result<DbPool, Box<dyn std::error::Error>> {
    let workspace_dir = get_workspace_dir();

    // Create workspace directory if it doesn't exist
    if !workspace_dir.exists() {
        fs::create_dir_all(&workspace_dir)?;
        info!("Created workspace directory: {}", workspace_dir.display());
    }

    let db_path = get_db_path();
    info!("Database path: {}", db_path.display());

    let manager = SqliteConnectionManager::file(&db_path);
    let pool = Pool::builder()
        .max_size(10)
        .build(manager)?;

    // Run schema migrations
    {
        let conn = pool.get()?;

        // Enable WAL mode for better concurrency
        conn.execute_batch("PRAGMA journal_mode=WAL;")?;
        conn.execute_batch("PRAGMA foreign_keys=ON;")?;

        // Create all tables
        schema::create_tables(&conn)?;
        info!("Database schema initialized successfully");
    }

    Ok(pool)
}
