//! Passkey sign-in, end to end with a software authenticator.

mod common;

use axum::http::StatusCode;
use common::{Client, SETUP_TOKEN, device};

#[tokio::test]
async fn a_fresh_server_asks_for_setup_and_locks_the_api() {
    let mut c = Client::new().await;
    let (_, status) = c.call("GET", "/api/auth/status", None).await;
    assert_eq!(status["needs_setup"], true);
    assert_eq!(status["signed_in"], false);
    let (code, body) = c.call("GET", "/api/due", None).await;
    assert_eq!(code, StatusCode::UNAUTHORIZED);
    assert_eq!(body["error"], "sign in first");
}

#[tokio::test]
async fn the_first_passkey_needs_the_setup_token() {
    let mut c = Client::new().await;
    let (code, _) = c.register(&mut device(), None).await;
    assert_eq!(code, StatusCode::FORBIDDEN);
    let (code, _) = c.register(&mut device(), Some("guess")).await;
    assert_eq!(code, StatusCode::FORBIDDEN);

    let (code, body) = c.register(&mut device(), Some(SETUP_TOKEN)).await;
    assert_eq!(code, StatusCode::OK, "{body}");
    assert_eq!(
        c.call("GET", "/api/due", None).await.0,
        StatusCode::OK,
        "registering signs you in"
    );
}

#[tokio::test]
async fn the_setup_token_stops_working_once_a_passkey_exists() {
    let first = Client::signed_in().await;
    let mut stranger = first.fresh_browser();
    let (code, _) = stranger.register(&mut device(), Some(SETUP_TOKEN)).await;
    assert_eq!(code, StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn sign_in_works_from_a_new_browser_with_the_same_device() {
    let mut phone = device();
    let mut c = Client::new().await;
    c.register(&mut phone, Some(SETUP_TOKEN)).await;

    let mut later = c.fresh_browser();
    assert_eq!(
        later.call("GET", "/api/due", None).await.0,
        StatusCode::UNAUTHORIZED
    );
    let (code, body) = later.sign_in(&mut phone).await;
    assert_eq!(code, StatusCode::OK, "{body}");
    assert_eq!(later.call("GET", "/api/due", None).await.0, StatusCode::OK);
}

#[tokio::test]
async fn an_unknown_device_cant_sign_in() {
    let c = Client::signed_in().await;
    let mut other = c.fresh_browser();
    let (code, _) = other.sign_in(&mut device()).await;
    assert_ne!(code, StatusCode::OK);
    assert_eq!(
        other.call("GET", "/api/due", None).await.0,
        StatusCode::UNAUTHORIZED
    );
}

#[tokio::test]
async fn a_signed_in_learner_can_add_a_second_device() {
    let mut c = Client::signed_in().await;
    let mut laptop = device();
    let (code, body) = c.register(&mut laptop, None).await;
    assert_eq!(code, StatusCode::OK, "{body}");

    let mut elsewhere = c.fresh_browser();
    assert_eq!(elsewhere.sign_in(&mut laptop).await.0, StatusCode::OK);
}

#[tokio::test]
async fn signing_out_locks_the_api_again() {
    let mut c = Client::signed_in().await;
    c.call("POST", "/api/auth/sign-out", None).await;
    assert_eq!(
        c.call("GET", "/api/due", None).await.0,
        StatusCode::UNAUTHORIZED
    );
}

#[tokio::test]
async fn finishing_without_starting_is_refused() {
    let mut c = Client::new().await;
    let (code, _) = c
        .call(
            "POST",
            "/api/auth/sign-in/finish",
            Some(serde_json::json!({})),
        )
        .await;
    assert!(code.is_client_error());
}
