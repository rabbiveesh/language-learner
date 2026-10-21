//! The API end to end, against an in-memory database.

use std::path::Path;

use axum::Router;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use serde_json::{Value, json};
use server::{Config, app, connect};
use tower::ServiceExt;

async fn test_app() -> Router {
    let config = Config {
        database_url: "sqlite::memory:".into(),
        port: 0,
        web_dist: "does-not-exist".into(),
    };
    app(connect(&config).await.unwrap(), Path::new("does-not-exist"))
}

async fn call(app: &Router, method: &str, uri: &str, body: Option<Value>) -> (StatusCode, Value) {
    let req = Request::builder()
        .method(method)
        .uri(uri)
        .header("content-type", "application/json");
    let req = req
        .body(body.map_or(Body::empty(), |b| Body::from(b.to_string())))
        .unwrap();
    let res = app.clone().oneshot(req).await.unwrap();
    let status = res.status();
    let bytes = res.into_body().collect().await.unwrap().to_bytes();
    (
        status,
        serde_json::from_slice(&bytes).unwrap_or(Value::Null),
    )
}

fn copo() -> Value {
    json!({
        "language": "pt-BR",
        "word": {
            "spelling": "copo", "ipa": "ˈkɔ.pu", "gender": "Masculine", "gloss": null,
            "personal": null, "image": "abc123", "audio": null
        }
    })
}

#[tokio::test]
async fn health_answers() {
    let app = test_app().await;
    let res = app
        .oneshot(Request::get("/api/health").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
}

#[tokio::test]
async fn a_new_word_with_a_picture_gets_picture_cards_and_they_show_up_as_new() {
    let app = test_app().await;
    let (status, created) = call(&app, "POST", "/api/notes", Some(copo())).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        created["cards"].as_array().unwrap().len(),
        3,
        "picture→word, word→picture, speaking"
    );

    let (_, due) = call(&app, "GET", "/api/due", None).await;
    assert_eq!(due.as_array().unwrap().len(), 3);
    assert!(
        due[0]["state"]["due"].is_null(),
        "new cards have no due date"
    );
}

#[tokio::test]
async fn remembering_a_card_takes_it_off_todays_list() {
    let app = test_app().await;
    let (_, created) = call(&app, "POST", "/api/notes", Some(copo())).await;
    let card_id = created["cards"][0]["id"].clone();

    let (status, state) = call(
        &app,
        "POST",
        "/api/reviews",
        Some(json!({ "card_id": card_id, "grade": "Good" })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(state["reps"], 1);

    let (_, due) = call(&app, "GET", "/api/due", None).await;
    assert!(
        due.as_array()
            .unwrap()
            .iter()
            .all(|c| c["card"]["id"] != card_id)
    );
}

#[tokio::test]
async fn reviewing_a_card_that_doesnt_exist_is_a_404() {
    let app = test_app().await;
    let (status, body) = call(
        &app,
        "POST",
        "/api/reviews",
        Some(json!({ "card_id": 99, "grade": "Good" })),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body["error"], "not found");
}

#[tokio::test]
async fn a_word_needs_a_spelling() {
    let app = test_app().await;
    let mut word = copo();
    word["word"]["spelling"] = json!("  ");
    let (status, _) = call(&app, "POST", "/api/notes", Some(word)).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}
