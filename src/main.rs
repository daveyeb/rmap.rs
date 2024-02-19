use std::{collections::VecDeque, env, sync::Arc, time::Duration};

use axum::{error_handling::HandleErrorLayer, middleware, BoxError, Router};
use dotenv::dotenv;
use futures::lock::Mutex;
use handlebars::Handlebars;
use lambda_http::{http::StatusCode, run};
use lambda_runtime::Error;
use middlewares::propagate_coop_coep_headers;
use state::AppState;
use tower::{buffer::BufferLayer, limit::RateLimitLayer, ServiceBuilder};
use tower_http::services::ServeDir;
use tower_sessions::{MemoryStore, SessionManagerLayer};

use crate::{state::Database, util::to_slice};

pub mod error;
pub mod middlewares;
pub mod routes;
pub mod services;
pub mod state;
pub mod util;

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing::info!("program starting");
    println!("program starting");
    setup_logging();

    let session_store = MemoryStore::default();
    let session_layer = SessionManagerLayer::new(session_store).with_name("rmap_session").with_secure(false);

    let state = init_state().await;
    let app = Router::new()
        .merge(services::back_auth(&state))
        .merge(services::back_api(&state))
        .layer(session_layer)
        .nest_service("/public", ServeDir::new("./public/"))
        .nest_service("/dist", ServeDir::new("./dist"))
        .nest_service("/pkg", ServeDir::new("./pkg"))
        .layer(middleware::from_fn(propagate_coop_coep_headers))
        .layer(
            ServiceBuilder::new()
                .layer(HandleErrorLayer::new(|err: BoxError| async move {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        format!("Unhandled error: {}", err),
                    )
                }))
                .layer(BufferLayer::new(1024))
                .layer(RateLimitLayer::new(5, Duration::from_secs(1))),
        );

    run(app).await
}

fn setup_logging() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        // disable printing the name of the module in every log line.
        .with_target(false)
        // disabling time is handy because CloudWatch will add the ingestion time.
        .without_time()
        .init();
}

async fn init_state() -> AppState<'static> {
    dotenv().ok();

    let config = aws_config::load_from_env().await;
    let client_s3 = aws_sdk_s3::Client::new(&config);
    let client_db = aws_sdk_dynamodb::Client::new(&config);

    let key = env::var("APP_KEY").unwrap();
    let client_id = env::var("CLIENT_ID").unwrap();
    let client_secret = env::var("CLIENT_SECRET").unwrap();

    // println!("key {}", key);
    let vkey = key
        .split_whitespace()
        .map(|s| s.parse().unwrap())
        .collect::<Vec<u8>>();

    let mut state = AppState {
        key: to_slice(&vkey),
        db: Arc::new(Mutex::new(Database {
            username: String::new(),
            search_link: String::new(),
            dash_repos: Vec::new(),
            search_repos: Vec::new(),
            scan_stats: Vec::new(),
            has_repos: true,
            ratelimit_count: 0,
            search_current: 0,
            nodes_items: VecDeque::new(),
            slots: vec![0, 1, 2, 3],
        })),
        hb: Handlebars::new(),
        client_id,
        client_secret,
        client_s3,
        client_db,
    };

    // let _ = state.hb.register_templates_directory(".hbs", "/tmp/views/");
    let _ = state.hb.register_templates_directory(".hbs", "./views/");

    state
}
