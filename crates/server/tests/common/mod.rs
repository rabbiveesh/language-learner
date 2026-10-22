//! A test client: the app over an in-memory database, a cookie jar, and a software passkey.

#![allow(dead_code)]

use std::collections::BTreeMap;
use std::path::Path;

use axum::Router;
use axum::body::Body;
use axum::http::{Request, StatusCode, header};
use http_body_util::BodyExt;
use serde_json::Value;
use server::{AuthConfig, Config, app, connect};
use tower::ServiceExt;
use webauthn_authenticator_rs::WebauthnAuthenticator;
use webauthn_authenticator_rs::softpasskey::SoftPasskey;

pub const ORIGIN: &str = "http://localhost:8080";
pub const SETUP_TOKEN: &str = "let-me-in";

pub struct Client {
    app: Router,
    cookies: BTreeMap<String, String>,
}

pub type Device = WebauthnAuthenticator<SoftPasskey>;

pub fn device() -> Device {
    WebauthnAuthenticator::new(SoftPasskey::new(true))
}

impl Client {
    pub async fn new() -> Self {
        let config = Config {
            database_url: "sqlite::memory:".into(),
            port: 0,
            web_dist: "does-not-exist".into(),
            auth: AuthConfig {
                origin: ORIGIN.parse().unwrap(),
                session_key: None,
                setup_token: Some(SETUP_TOKEN.into()),
            },
        };
        let app = app(connect(&config).await.unwrap(), Path::new("does-not-exist"));
        Client {
            app,
            cookies: BTreeMap::new(),
        }
    }

    /// The same server, but a browser with no cookies.
    pub fn fresh_browser(&self) -> Self {
        Client {
            app: self.app.clone(),
            cookies: BTreeMap::new(),
        }
    }

    pub async fn call(
        &mut self,
        method: &str,
        uri: &str,
        body: Option<Value>,
    ) -> (StatusCode, Value) {
        let mut req = Request::builder()
            .method(method)
            .uri(uri)
            .header("content-type", "application/json");
        if !self.cookies.is_empty() {
            let jar: Vec<String> = self
                .cookies
                .iter()
                .map(|(k, v)| format!("{k}={v}"))
                .collect();
            req = req.header(header::COOKIE, jar.join("; "));
        }
        let req = req
            .body(body.map_or(Body::empty(), |b| Body::from(b.to_string())))
            .unwrap();
        let res = self.app.clone().oneshot(req).await.unwrap();
        for set in res.headers().get_all(header::SET_COOKIE) {
            let set = set.to_str().unwrap();
            let (pair, attrs) = set.split_once(';').unwrap_or((set, ""));
            let (name, value) = pair.split_once('=').unwrap();
            if value.is_empty() || attrs.contains("Max-Age=0") {
                self.cookies.remove(name);
            } else {
                self.cookies.insert(name.into(), value.into());
            }
        }
        let status = res.status();
        let bytes = res.into_body().collect().await.unwrap().to_bytes();
        (
            status,
            serde_json::from_slice(&bytes).unwrap_or(Value::Null),
        )
    }

    /// Register `device` as a passkey. Returns the finish call's status and body.
    pub async fn register(
        &mut self,
        device: &mut Device,
        setup_token: Option<&str>,
    ) -> (StatusCode, Value) {
        let (status, challenge) = self
            .call(
                "POST",
                "/api/auth/register/start",
                Some(serde_json::json!({ "setup_token": setup_token })),
            )
            .await;
        if status != StatusCode::OK {
            return (status, challenge);
        }
        let credential = device
            .do_registration(
                ORIGIN.parse().unwrap(),
                serde_json::from_value(challenge).unwrap(),
            )
            .unwrap();
        self.call(
            "POST",
            "/api/auth/register/finish",
            Some(serde_json::to_value(credential).unwrap()),
        )
        .await
    }

    /// Sign in with `device`. Returns the finish call's status and body.
    pub async fn sign_in(&mut self, device: &mut Device) -> (StatusCode, Value) {
        let (status, challenge) = self.call("POST", "/api/auth/sign-in/start", None).await;
        if status != StatusCode::OK {
            return (status, challenge);
        }
        match device.do_authentication(
            ORIGIN.parse().unwrap(),
            serde_json::from_value(challenge).unwrap(),
        ) {
            Ok(credential) => {
                self.call(
                    "POST",
                    "/api/auth/sign-in/finish",
                    Some(serde_json::to_value(credential).unwrap()),
                )
                .await
            }
            Err(e) => (StatusCode::UNAUTHORIZED, Value::String(format!("{e:?}"))),
        }
    }

    /// A client already signed in with a first passkey.
    pub async fn signed_in() -> Self {
        let mut client = Client::new().await;
        let (status, _) = client.register(&mut device(), Some(SETUP_TOKEN)).await;
        assert_eq!(status, StatusCode::OK);
        client
    }
}
