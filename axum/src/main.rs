use axum::{ 
    extract::Json as JsonExtractor, 
    extract::Query, 
    http::StatusCode, 
    response::IntoResponse, 
    routing::{get,post },
    Json, 
    Router
};
use serde_json::json;
use lazy_static::lazy_static;
use reverse_geocoder::ReverseGeocoder;
use serde::Deserialize;
use tokio::net::TcpListener;

lazy_static! {
    static ref GEOCODER: ReverseGeocoder = ReverseGeocoder::new();
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let app = Router::new()
                        .route("/", get(query))
                        .route("/batch", post(query_multiple));
    let addr = "127.0.0.1:3000";

    let listener = TcpListener::bind(addr).await.unwrap();

    tracing::debug!("listening on {}", addr);

    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}

#[derive(Debug, Deserialize)]
struct Location {
    lat: f64,
    long: f64,
}

#[derive(Debug, Deserialize)]
struct Locations {
    locations: Vec<Location>,
}

async fn query(Query(params): Query<Location>) -> impl IntoResponse {
    let loc = GEOCODER.search((params.lat, params.long));
    (StatusCode::OK, Json(loc))
}

async fn query_multiple(JsonExtractor(params): JsonExtractor<Locations>) -> impl IntoResponse {
    if params.locations.len() > 100 {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "Too many locations. Maximum allowed is 100." }))
        );
    }

    let results = params
        .locations
        .iter()
        .map(|loc| GEOCODER.search((loc.lat, loc.long)))
        .collect::<Vec<_>>();
    
        (StatusCode::OK, Json(json!({ "results": results })))
}
