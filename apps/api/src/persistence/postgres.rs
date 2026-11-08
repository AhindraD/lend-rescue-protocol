use std::time::Duration;

use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use tracing::{info, warn};

use crate::error::ApiError;

/// Wrapper around optional PostgreSQL connection pool.
#[derive(Debug, Clone)]
pub struct DatabasePool {
    pool: Option<PgPool>,
}

impl DatabasePool {
    /// Creates an in-memory or disabled database pool.
    pub fn disabled() -> Self {
        Self { pool: None }
    }

    /// Initializes PostgreSQL pool from URL and runs pending migrations.
    pub async fn init(database_url: Option<&str>, demo_mode: bool) -> Result<Self, ApiError> {
        let url = match database_url {
            Some(u) if !u.trim().is_empty() => u,
            _ => {
                info!("No DATABASE_URL configured; running with in-memory persistence fallback");
                return Ok(Self::disabled());
            }
        };

        match PgPoolOptions::new()
            .max_connections(10)
            .acquire_timeout(Duration::from_secs(3))
            .connect(url)
            .await
        {
            Ok(pool) => {
                info!("Connected to PostgreSQL database");

                // Run migrations
                match sqlx::migrate!("./migrations").run(&pool).await {
                    Ok(_) => info!("Database migrations applied successfully"),
                    Err(e) => {
                        warn!("Database migration error: {}", e);
                        if !demo_mode {
                            return Err(ApiError::Database(format!("Migration failed: {}", e)));
                        }
                    }
                }

                Ok(Self { pool: Some(pool) })
            }
            Err(e) => {
                warn!("Could not connect to PostgreSQL at {}: {}", url, e);
                if demo_mode {
                    info!("Demo mode enabled; proceeding without active PostgreSQL connection");
                    Ok(Self::disabled())
                } else {
                    Err(ApiError::Database(format!(
                        "Failed to connect to database: {}",
                        e
                    )))
                }
            }
        }
    }

    pub fn is_connected(&self) -> bool {
        self.pool.is_some()
    }

    pub fn pool(&self) -> Option<&PgPool> {
        self.pool.as_ref()
    }
}
