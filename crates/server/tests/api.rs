//! The API end to end, against an in-memory database.

mod common;

use axum::http::StatusCode;
use common::Client;
use serde_json::{Value, json};

async fn call(c: &mut Client, method: &str, uri: &str, body: Option<Value>) -> (StatusCode, Value) {
    c.call(method, uri, body).await
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
    let mut signed_out = Client::new().await;
    assert_eq!(
        call(&mut signed_out, "GET", "/api/health", None).await.0,
        StatusCode::OK
    );
}

#[tokio::test]
async fn a_new_word_with_a_picture_gets_picture_cards_and_they_show_up_as_new() {
    let mut app = Client::signed_in().await;
    let (status, created) = call(&mut app, "POST", "/api/notes", Some(copo())).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        created["cards"].as_array().unwrap().len(),
        3,
        "picture→word, word→picture, speaking"
    );

    let (_, due) = call(&mut app, "GET", "/api/due", None).await;
    assert_eq!(due.as_array().unwrap().len(), 3);
    assert!(
        due[0]["state"]["due"].is_null(),
        "new cards have no due date"
    );
}

#[tokio::test]
async fn remembering_a_card_takes_it_off_todays_list() {
    let mut app = Client::signed_in().await;
    let (_, created) = call(&mut app, "POST", "/api/notes", Some(copo())).await;
    let card_id = created["cards"][0]["id"].clone();

    let (status, state) = call(
        &mut app,
        "POST",
        "/api/reviews",
        Some(json!({ "card_id": card_id, "grade": "Good" })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(state["reps"], 1);

    let (_, due) = call(&mut app, "GET", "/api/due", None).await;
    assert!(
        due.as_array()
            .unwrap()
            .iter()
            .all(|c| c["card"]["id"] != card_id)
    );
}

#[tokio::test]
async fn reviewing_a_card_that_doesnt_exist_is_a_404() {
    let mut app = Client::signed_in().await;
    let (status, body) = call(
        &mut app,
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
    let mut app = Client::signed_in().await;
    let mut word = copo();
    word["word"]["spelling"] = json!("  ");
    let (status, _) = call(&mut app, "POST", "/api/notes", Some(word)).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}
