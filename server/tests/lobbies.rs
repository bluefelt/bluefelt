use bluefelt_core::*;
use axum::http::StatusCode;
use tower::ServiceExt;  // for `oneshot`
use axum::Router;
use bluefelt_core::{create_lobby, list_lobbies};

#[tokio::test]
async fn create_and_list() {
    let lobbies = std::sync::Arc::new(LobbyMap::new());
    let app = Router::new()
        .route("/lobbies", axum::routing::post(create_lobby))
        .route("/lobbies", axum::routing::get(list_lobbies))
        .with_state(lobbies);

    // create lobby
    let resp = app
        .clone()
        .oneshot(axum::http::Request::builder()
            .method("POST")
            .uri("/lobbies")
            .header("content-type", "application/json")
            .body(r#"{"gameId":"love-letter"}"#.into())
            .unwrap())
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // list
    let resp = app
        .oneshot(axum::http::Request::builder()
            .method("GET")
            .uri("/lobbies")
            .body("".into())
            .unwrap())
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}

// tests won't work with the current implementation
// need to define methods in lib.rs