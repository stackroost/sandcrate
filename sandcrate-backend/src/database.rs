use sqlx::{PgPool, PgPoolOptions, postgres::PgPoolOptions as PgPoolOptionsPostgres};
use std::time::Duration;
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub min_connections: u32,
    pub connect_timeout: Duration,
    pub idle_timeout: Duration,
    pub max_lifetime: Duration,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            url: std::env::var("DATABASE_URL")
                .unwrap_or_else(|_| "postgresql://sandcrate:sandcrate@localhost:5432/sandcrate".to_string()),
            max_connections: 10,
            min_connections: 2,
            connect_timeout: Duration::from_secs(30),
            idle_timeout: Duration::from_secs(300),
            max_lifetime: Duration::from_secs(3600),
        }
    }
}

pub async fn create_pool(config: &DatabaseConfig) -> Result<PgPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(config.max_connections)
        .min_connections(config.min_connections)
        .connect_timeout(config.connect_timeout)
        .idle_timeout(config.idle_timeout)
        .max_lifetime(config.max_lifetime)
        .connect(&config.url)
        .await
}


#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Plugin {
    pub id: Uuid,
    pub name: String,
    pub filename: String,
    pub file_path: String,
    pub file_size: i64,
    pub description: Option<String>,
    pub version: String,
    pub author: Option<String>,
    pub tags: Vec<String>,
    pub status: PluginStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_executed_at: Option<DateTime<Utc>>,
    pub execution_count: i32,
    pub average_execution_time_ms: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "plugin_status", rename_all = "lowercase")]
pub enum PluginStatus {
    Active,
    Inactive,
    Error,
    Processing,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct PluginExecution {
    pub id: Uuid,
    pub plugin_id: Uuid,
    pub user_id: Option<Uuid>,
    pub session_id: Option<String>,
    pub parameters: Option<serde_json::Value>,
    pub result: Option<String>,
    pub error: Option<String>,
    pub execution_time_ms: i64,
    pub status: ExecutionStatus,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "execution_status", rename_all = "lowercase")]
pub enum ExecutionStatus {
    Running,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: Option<String>,
    pub name: String,
    pub role: UserRole,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_login_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "user_role", rename_all = "lowercase")]
pub enum UserRole {
    Admin,
    User,
    Guest,
}

#[async_trait::async_trait]
pub trait PluginRepository {
    async fn create_plugin(&self, plugin: CreatePluginRequest) -> Result<Plugin, sqlx::Error>;
    async fn get_plugin_by_id(&self, id: Uuid) -> Result<Option<Plugin>, sqlx::Error>;
    async fn get_plugin_by_filename(&self, filename: &str) -> Result<Option<Plugin>, sqlx::Error>;
    async fn list_plugins(&self, limit: Option<i64>, offset: Option<i64>) -> Result<Vec<Plugin>, sqlx::Error>;
    async fn update_plugin(&self, id: Uuid, updates: UpdatePluginRequest) -> Result<Plugin, sqlx::Error>;
    async fn delete_plugin(&self, id: Uuid) -> Result<bool, sqlx::Error>;
    async fn record_execution(&self, execution: CreateExecutionRequest) -> Result<PluginExecution, sqlx::Error>;
    async fn get_execution_history(&self, plugin_id: Uuid, limit: Option<i64>) -> Result<Vec<PluginExecution>, sqlx::Error>;
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreatePluginRequest {
    pub name: String,
    pub filename: String,
    pub file_path: String,
    pub file_size: i64,
    pub description: Option<String>,
    pub version: String,
    pub author: Option<String>,
    pub tags: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdatePluginRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub version: Option<String>,
    pub author: Option<String>,
    pub tags: Option<Vec<String>>,
    pub status: Option<PluginStatus>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateExecutionRequest {
    pub plugin_id: Uuid,
    pub user_id: Option<Uuid>,
    pub session_id: Option<String>,
    pub parameters: Option<serde_json::Value>,
}

pub struct PostgresPluginRepository {
    pool: PgPool,
}

impl PostgresPluginRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl PluginRepository for PostgresPluginRepository {
    async fn create_plugin(&self, plugin: CreatePluginRequest) -> Result<Plugin, sqlx::Error> {
        let id = Uuid::new_v4();
        let now = Utc::now();
        
        sqlx::query_as!(
            Plugin,
            r#"
            INSERT INTO plugins (id, name, filename, file_path, file_size, description, version, author, tags, status, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            RETURNING *
            "#,
            id,
            plugin.name,
            plugin.filename,
            plugin.file_path,
            plugin.file_size,
            plugin.description,
            plugin.version,
            plugin.author,
            &plugin.tags,
            PluginStatus::Active as PluginStatus,
            now,
            now
        )
        .fetch_one(&self.pool)
        .await
    }

    async fn get_plugin_by_id(&self, id: Uuid) -> Result<Option<Plugin>, sqlx::Error> {
        sqlx::query_as!(
            Plugin,
            "SELECT * FROM plugins WHERE id = $1",
            id
        )
        .fetch_optional(&self.pool)
        .await
    }

    async fn get_plugin_by_filename(&self, filename: &str) -> Result<Option<Plugin>, sqlx::Error> {
        sqlx::query_as!(
            Plugin,
            "SELECT * FROM plugins WHERE filename = $1",
            filename
        )
        .fetch_optional(&self.pool)
        .await
    }

    async fn list_plugins(&self, limit: Option<i64>, offset: Option<i64>) -> Result<Vec<Plugin>, sqlx::Error> {
        let limit = limit.unwrap_or(100);
        let offset = offset.unwrap_or(0);
        
        sqlx::query_as!(
            Plugin,
            "SELECT * FROM plugins ORDER BY created_at DESC LIMIT $1 OFFSET $2",
            limit,
            offset
        )
        .fetch_all(&self.pool)
        .await
    }

    async fn update_plugin(&self, id: Uuid, updates: UpdatePluginRequest) -> Result<Plugin, sqlx::Error> {
        let now = Utc::now();
        
        let mut query = String::from("UPDATE plugins SET updated_at = $1");
        let mut params: Vec<Box<dyn sqlx::Encode<'_, sqlx::Postgres> + Send + Sync>> = vec![Box::new(now)];
        let mut param_count = 1;

        if let Some(name) = updates.name {
            param_count += 1;
            query.push_str(&format!(", name = ${}", param_count));
            params.push(Box::new(name));
        }

        if let Some(description) = updates.description {
            param_count += 1;
            query.push_str(&format!(", description = ${}", param_count));
            params.push(Box::new(description));
        }

        if let Some(version) = updates.version {
            param_count += 1;
            query.push_str(&format!(", version = ${}", param_count));
            params.push(Box::new(version));
        }

        if let Some(author) = updates.author {
            param_count += 1;
            query.push_str(&format!(", author = ${}", param_count));
            params.push(Box::new(author));
        }

        if let Some(tags) = updates.tags {
            param_count += 1;
            query.push_str(&format!(", tags = ${}", param_count));
            params.push(Box::new(tags));
        }

        if let Some(status) = updates.status {
            param_count += 1;
            query.push_str(&format!(", status = ${}", param_count));
            params.push(Box::new(status));
        }

        param_count += 1;
        query.push_str(&format!(" WHERE id = ${} RETURNING *", param_count));
        params.push(Box::new(id));

        sqlx::query_as::<_, Plugin>(&query)
            .bind_all(params)
            .fetch_one(&self.pool)
            .await
    }

    async fn delete_plugin(&self, id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query!(
            "DELETE FROM plugins WHERE id = $1",
            id
        )
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    async fn record_execution(&self, execution: CreateExecutionRequest) -> Result<PluginExecution, sqlx::Error> {
        let id = Uuid::new_v4();
        let now = Utc::now();
        
        sqlx::query_as!(
            PluginExecution,
            r#"
            INSERT INTO plugin_executions (id, plugin_id, user_id, session_id, parameters, status, started_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING *
            "#,
            id,
            execution.plugin_id,
            execution.user_id,
            execution.session_id,
            execution.parameters,
            ExecutionStatus::Running as ExecutionStatus,
            now
        )
        .fetch_one(&self.pool)
        .await
    }

    async fn get_execution_history(&self, plugin_id: Uuid, limit: Option<i64>) -> Result<Vec<PluginExecution>, sqlx::Error> {
        let limit = limit.unwrap_or(50);
        
        sqlx::query_as!(
            PluginExecution,
            "SELECT * FROM plugin_executions WHERE plugin_id = $1 ORDER BY started_at DESC LIMIT $2",
            plugin_id,
            limit
        )
        .fetch_all(&self.pool)
        .await
    }
} 