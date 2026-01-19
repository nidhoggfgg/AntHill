use axum::Router;
use tower_http::cors::{Any, CorsLayer};

pub fn add_cors(router: Router) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([
            axum::http::Method::GET,
            axum::http::Method::POST,
            axum::http::Method::PUT,
            axum::http::Method::DELETE,
            axum::http::Method::PATCH,
        ])
        .allow_headers(Any);

    router.layer(cors)
}
