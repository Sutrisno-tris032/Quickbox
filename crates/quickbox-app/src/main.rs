use anyhow::Result;
use quickbox_api::{create_app, AppState};
use quickbox_core::db::init_db;
use quickbox_core::retention::run_cleanup_cycle;
use quickbox_smtp::ingest::NewMailNotification;
use quickbox_smtp::SmtpServer;
use std::env;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::broadcast;
use tower_http::services::{ServeDir, ServeFile};
use tracing::{error, info};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<()> {
    // 1. Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info,quickbox=debug")),
        )
        .init();

    info!("Starting Quick-Box v{} (Monolith)", quickbox_core::version());

    // 2. Load configurations from environment or defaults
    let database_url = env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite://quickbox.db".to_string());
    let attachments_dir = PathBuf::from(env::var("ATTACHMENTS_DIR").unwrap_or_else(|_| "./data/attachments".to_string()));
    let domain = env::var("DOMAIN").unwrap_or_else(|_| "quickbox.dev".to_string());
    let smtp_port: u16 = env::var("SMTP_PORT").unwrap_or_else(|_| "25".to_string()).parse()?;
    let http_port: u16 = env::var("HTTP_PORT").unwrap_or_else(|_| "8080".to_string()).parse()?;
    let jwt_secret = env::var("JWT_SECRET").unwrap_or_else(|_| "quickbox-default-insecure-secret-key-change-in-prod".to_string());
    let retention_days: i64 = env::var("RETENTION_DAYS").unwrap_or_else(|_| "30".to_string()).parse()?;
    let frontend_dist = PathBuf::from(env::var("FRONTEND_DIST").unwrap_or_else(|_| "./frontend/dist".to_string()));
    // Nonaktif (endpoint /internal/inbound -> 404) kecuali operator menyetelnya secara eksplisit.
    let inbound_secret = env::var("INBOUND_SECRET").ok();
    let enable_smtp = env::var("ENABLE_SMTP").unwrap_or_else(|_| "true".to_string()) != "false";

    tokio::fs::create_dir_all(&attachments_dir).await?;

    // 3. Connect to Database
    info!("Connecting to SQLite database: {}", database_url);
    let pool = init_db(&database_url).await?;

    // 4. Create notification broadcast channel
    let (tx, _) = broadcast::channel::<NewMailNotification>(1024);
    let broadcast_tx = Arc::new(tx);

    // 5. Start Background Retention Worker (Runs once per hour)
    let retention_pool = pool.clone();
    let retention_dir = attachments_dir.clone();
    tokio::spawn(async move {
        info!("Retention worker started (interval: 1 hour, retention: {} days)", retention_days);
        loop {
            if let Err(e) = run_cleanup_cycle(&retention_pool, &retention_dir, retention_days).await {
                error!("Error running retention cleanup: {}", e);
            }
            tokio::time::sleep(Duration::from_secs(3600)).await;
        }
    });

    // 6. Start Inbound SMTP Server (can be disabled when using Cloudflare Email Worker)
    if enable_smtp {
        let smtp_bind = format!("0.0.0.0:{}", smtp_port);
        let smtp_server = SmtpServer::bind(&smtp_bind, &domain, 10_000_000)
            .await?
            .with_storage(pool.clone(), attachments_dir.clone(), broadcast_tx.clone());

        tokio::spawn(async move {
            info!("Inbound SMTP Server listening on {}", smtp_bind);
            if let Err(e) = smtp_server.serve().await {
                error!("SMTP Server error: {}", e);
            }
        });
    } else {
        info!("SMTP listener disabled (ENABLE_SMTP=false). Using Cloudflare Email Worker webhook instead.");
    }

    // 7. Setup Axum Web API & Static Frontend Server
    let state = AppState {
        pool,
        attachments_dir,
        broadcast_tx,
        jwt_secret,
        domain,
        inbound_secret,
    };

    let api_app = create_app(state);

    // If frontend dist directory exists, serve static files with SPA fallback
    let app = if frontend_dist.exists() {
        info!("Serving frontend static assets from: {}", frontend_dist.display());
        let index_file = frontend_dist.join("index.html");
        let serve_dir = ServeDir::new(&frontend_dist).fallback(ServeFile::new(index_file));
        api_app.fallback_service(serve_dir)
    } else {
        info!("Frontend dist folder not found ({}), serving API only", frontend_dist.display());
        api_app
    };

    let http_bind = format!("0.0.0.0:{}", http_port);
    info!("Quick-Box Web UI & API listening on http://{}", http_bind);
    let listener = tokio::net::TcpListener::bind(&http_bind).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
