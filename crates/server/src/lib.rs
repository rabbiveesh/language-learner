//! The personal server: a JSON API over SQLite, plus the built web app. Everything under
//! `/api` except health and sign-in needs a passkey sign-in (see [`auth`]).

mod api;
pub mod auth;
mod error;
mod store;

use std::path::{Path, PathBuf};
use std::sync::Arc;

use axum::Router;
use domain::Scheduler;
use sqlx::SqlitePool;
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions};
use tower_http::services::{ServeDir, ServeFile};
use tower_http::trace::TraceLayer;

pub use api::{DueCard, NewReview, NewWordNote, NoteWithCards};
pub use auth::{Auth, AuthConfig};

/// Settings from the environment.
#[derive(Debug, Clone)]
pub struct Config {
    /// `DATABASE_URL`, default `sqlite://data/fala.db`.
    pub database_url: String,
    /// `PORT`, default 8080.
    pub port: u16,
    /// `WEB_DIST`, the built web app, default `web/dist`.
    pub web_dist: PathBuf,
    pub auth: AuthConfig,
}

impl Config {
    pub fn from_env() -> Result<Self, String> {
        let var = |k: &str| std::env::var(k).ok().filter(|v| !v.is_empty());
        Ok(Config {
            database_url: var("DATABASE_URL").unwrap_or_else(|| "sqlite://data/fala.db".into()),
            port: var("PORT")
                .map(|p| p.parse().map_err(|e| format!("PORT: {e}")))
                .transpose()?
                .unwrap_or(8080),
            web_dist: var("WEB_DIST").unwrap_or_else(|| "web/dist".into()).into(),
            auth: AuthConfig {
                origin: var("PUBLIC_ORIGIN")
                    .unwrap_or_else(|| "http://localhost:8080".into())
                    .parse()
                    .map_err(|e| format!("PUBLIC_ORIGIN: {e}"))?,
                session_key: var("SESSION_KEY"),
                setup_token: var("SETUP_TOKEN"),
            },
        })
    }
}

#[derive(Clone)]
pub struct AppState {
    pub db: SqlitePool,
    pub scheduler: Arc<Scheduler>,
    pub auth: Arc<Auth>,
}

/// Open (creating if needed) and migrate the database.
pub async fn connect(config: &Config) -> Result<AppState, Box<dyn std::error::Error>> {
    let options: SqliteConnectOptions = config.database_url.parse()?;
    // WAL is what Litestream replicates; the busy timeout covers its checkpoints.
    let options = options
        .create_if_missing(true)
        .foreign_keys(true)
        .journal_mode(SqliteJournalMode::Wal)
        .busy_timeout(std::time::Duration::from_secs(5));
    if let Some(dir) = options
        .get_filename()
        .parent()
        .filter(|d| !d.as_os_str().is_empty())
    {
        std::fs::create_dir_all(dir)?;
    }
    // An in-memory database exists per connection, so keep to one there.
    let in_memory = config.database_url.contains(":memory:");
    let db = SqlitePoolOptions::new()
        .max_connections(if in_memory { 1 } else { 4 })
        .connect_with(options)
        .await?;
    sqlx::migrate!().run(&db).await?;
    // TODO(design): FSRS parameters fitted to the learner's own log once there are enough
    // reviews, and a desired retention the learner can choose.
    let scheduler = Scheduler::new(&[], 0.9)?;
    Ok(AppState {
        db,
        scheduler: Arc::new(scheduler),
        auth: Arc::new(Auth::new(&config.auth)?),
    })
}

pub fn app(state: AppState, web_dist: &Path) -> Router {
    let web = ServeDir::new(web_dist).fallback(ServeFile::new(web_dist.join("index.html")));
    Router::new()
        .nest("/api", api::routes(state.clone()))
        .with_state(state)
        .fallback_service(web)
        .layer(TraceLayer::new_for_http())
}
