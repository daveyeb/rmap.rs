use axum::{
    routing::{get, post},
    Router,
};

use crate::{
    routes::{
        authorize, callback, destroy, get_blob, get_dashboard, get_repo, get_scan, home_page, pagination, post_dashboard, search
    },
    state::AppState,
};

pub fn back_auth(state: AppState<'static>) -> Router {
    Router::new()
        // .route("/auth/session", get(routes::session::data_handler)) // gets session data
        .route("/", get(home_page))
        .route("/auth/destroy", get(destroy)) // deletes username in session
        .route("/auth/authorize", get(authorize)) // sets username in session
        .route("/auth/callback", get(callback)) // deletes username in session
        .with_state(state.to_owned())
}

pub fn back_api(state: AppState<'static>) -> Router {
    Router::new()
        // .route("/auth/session", get(routes::session::data_handler)) // gets session data
        .route("/blob", get(get_blob))
        .route("/dashboard", get(get_dashboard))
        .route("/dashboard", post(post_dashboard))
        .route("/search", post(search))
        .route("/repos", get(get_repo))
        .route("/scans", post(get_scan))
        .route("/search/page", get(pagination))
        .with_state(state.to_owned())
}
