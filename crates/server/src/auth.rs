//! Sign-in with passkeys, for the one learner this server belongs to.
//!
//! - The first passkey needs the one-time `SETUP_TOKEN` (a Fly secret). Once a passkey exists,
//!   the token stops working.
//! - More passkeys (another device) can be added only while signed in.
//! - Signing in sets an encrypted cookie that lasts [`SESSION_DAYS`]. WebAuthn ceremony state
//!   travels between "start" and "finish" in a second, short-lived encrypted cookie, so the
//!   server keeps no session table.

use axum::extract::{FromRef, Request, State};
use axum::middleware::Next;
use axum::response::Response;
use axum::routing::{get, post};
use axum::{Json, Router};
use axum_extra::extract::PrivateCookieJar;
use axum_extra::extract::cookie::{Cookie, Key, SameSite};
use jiff::{SignedDuration, Timestamp};
use serde::{Deserialize, Serialize};
use sqlx::{Row, SqlitePool};
use ts_rs::TS;
use webauthn_rs::prelude::*;

use crate::AppState;
use crate::error::ApiError;

pub const SESSION_DAYS: i64 = 30;
const CEREMONY_MINUTES: i64 = 5;
const SESSION_COOKIE: &str = "fala_session";
const CEREMONY_COOKIE: &str = "fala_ceremony";

/// Sign-in settings from the environment.
#[derive(Debug, Clone)]
pub struct AuthConfig {
    /// `PUBLIC_ORIGIN`: where the app is served, e.g. `https://fala.veesh.me`. Passkeys are
    /// bound to its host. Default `http://localhost:8080` (any localhost port works in dev).
    pub origin: Url,
    /// `SESSION_KEY`: at least 64 bytes (`openssl rand -base64 64`). Optional on localhost, where
    /// a random key is made at startup (sign-ins then don't survive a restart).
    pub session_key: Option<String>,
    /// `SETUP_TOKEN`: lets the first passkey be registered.
    pub setup_token: Option<String>,
}

pub struct Auth {
    webauthn: Webauthn,
    key: Key,
    setup_token: Option<String>,
    secure: bool,
}

impl Auth {
    pub fn new(config: &AuthConfig) -> Result<Self, String> {
        let host = config
            .origin
            .host_str()
            .ok_or("PUBLIC_ORIGIN has no host")?;
        let localhost = host == "localhost";
        let webauthn = WebauthnBuilder::new(host, &config.origin)
            .map_err(|e| format!("PUBLIC_ORIGIN: {e}"))?
            .rp_name("Fala")
            .allow_any_port(localhost)
            .build()
            .map_err(|e| format!("passkeys: {e}"))?;
        let key = match &config.session_key {
            Some(k) => {
                Key::try_from(k.as_bytes()).map_err(|_| "SESSION_KEY must be at least 64 bytes")?
            }
            None if localhost => {
                tracing::warn!("no SESSION_KEY: using a random one, sign-ins end on restart");
                Key::generate()
            }
            None => return Err("SESSION_KEY is required outside localhost".into()),
        };
        Ok(Auth {
            webauthn,
            key,
            setup_token: config.setup_token.clone(),
            secure: config.origin.scheme() == "https",
        })
    }

    fn cookie(&self, name: &'static str, value: String, minutes: i64) -> Cookie<'static> {
        Cookie::build((name, value))
            .path("/")
            .http_only(true)
            .secure(self.secure)
            .same_site(SameSite::Strict)
            .max_age(cookie::time::Duration::minutes(minutes))
            .build()
    }

    fn setup_token_matches(&self, given: Option<&str>) -> bool {
        match (&self.setup_token, given) {
            (Some(expected), Some(given)) => {
                constant_time_eq::constant_time_eq(expected.as_bytes(), given.as_bytes())
            }
            _ => false,
        }
    }
}

impl FromRef<AppState> for Key {
    fn from_ref(state: &AppState) -> Key {
        state.auth.key.clone()
    }
}

#[derive(Serialize, Deserialize)]
struct Session {
    expires: Timestamp,
}

#[derive(Serialize, Deserialize)]
enum Ceremony {
    Register(PasskeyRegistration),
    SignIn(PasskeyAuthentication),
}

#[derive(Serialize, Deserialize)]
struct Pending {
    ceremony: Ceremony,
    expires: Timestamp,
}

fn signed_in(jar: &PrivateCookieJar) -> bool {
    jar.get(SESSION_COOKIE)
        .and_then(|c| serde_json::from_str::<Session>(c.value()).ok())
        .is_some_and(|s| s.expires > Timestamp::now())
}

fn take_ceremony(jar: PrivateCookieJar) -> (PrivateCookieJar, Option<Ceremony>) {
    let pending = jar
        .get(CEREMONY_COOKIE)
        .and_then(|c| serde_json::from_str::<Pending>(c.value()).ok())
        .filter(|p| p.expires > Timestamp::now())
        .map(|p| p.ceremony);
    (
        jar.remove(Cookie::build(CEREMONY_COOKIE).path("/")),
        pending,
    )
}

fn start_ceremony(
    auth: &Auth,
    jar: PrivateCookieJar,
    ceremony: Ceremony,
) -> Result<PrivateCookieJar, ApiError> {
    let expires = Timestamp::now() + SignedDuration::from_mins(CEREMONY_MINUTES);
    let value = serde_json::to_string(&Pending { ceremony, expires })?;
    Ok(jar.add(auth.cookie(CEREMONY_COOKIE, value, CEREMONY_MINUTES)))
}

fn sign_in(auth: &Auth, jar: PrivateCookieJar) -> Result<PrivateCookieJar, ApiError> {
    let expires = Timestamp::now() + SignedDuration::from_hours(24 * SESSION_DAYS);
    let value = serde_json::to_string(&Session { expires })?;
    Ok(jar.add(auth.cookie(SESSION_COOKIE, value, 24 * 60 * SESSION_DAYS)))
}

/// Middleware: everything it wraps needs a signed-in learner.
pub async fn require_session(
    jar: PrivateCookieJar,
    req: Request,
    next: Next,
) -> Result<Response, ApiError> {
    if signed_in(&jar) {
        Ok(next.run(req).await)
    } else {
        Err(ApiError::Unauthorized)
    }
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/status", get(status))
        .route("/register/start", post(register_start))
        .route("/register/finish", post(register_finish))
        .route("/sign-in/start", post(sign_in_start))
        .route("/sign-in/finish", post(sign_in_finish))
        .route("/sign-out", post(sign_out))
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export)]
pub struct AuthStatus {
    pub signed_in: bool,
    /// No passkey yet: the app should ask for the setup token.
    pub needs_setup: bool,
}

async fn status(
    State(s): State<AppState>,
    jar: PrivateCookieJar,
) -> Result<Json<AuthStatus>, ApiError> {
    let needs_setup = passkeys(&s.db).await?.is_empty();
    Ok(Json(AuthStatus {
        signed_in: signed_in(&jar),
        needs_setup,
    }))
}

#[derive(Debug, Clone, Default, Deserialize, TS)]
#[ts(export)]
pub struct RegisterStart {
    /// Needed only for the first passkey.
    pub setup_token: Option<String>,
}

async fn register_start(
    State(s): State<AppState>,
    jar: PrivateCookieJar,
    Json(body): Json<RegisterStart>,
) -> Result<(PrivateCookieJar, Json<CreationChallengeResponse>), ApiError> {
    let existing = passkeys(&s.db).await?;
    let allowed = signed_in(&jar)
        || (existing.is_empty() && s.auth.setup_token_matches(body.setup_token.as_deref()));
    if !allowed {
        return Err(if existing.is_empty() && s.auth.setup_token.is_none() {
            ApiError::Forbidden("the server has no SETUP_TOKEN set".into())
        } else {
            ApiError::Forbidden("sign in, or use the setup link, to add a passkey".into())
        });
    }
    let user_id = owner(&s.db).await?;
    let exclude = existing.iter().map(|p| p.cred_id().clone()).collect();
    let (challenge, state) =
        s.auth
            .webauthn
            .start_passkey_registration(user_id, "me", "Fala learner", Some(exclude))?;
    Ok((
        start_ceremony(&s.auth, jar, Ceremony::Register(state))?,
        Json(challenge),
    ))
}

async fn register_finish(
    State(s): State<AppState>,
    jar: PrivateCookieJar,
    Json(credential): Json<RegisterPublicKeyCredential>,
) -> Result<(PrivateCookieJar, Json<AuthStatus>), ApiError> {
    let was_signed_in = signed_in(&jar);
    let (jar, ceremony) = take_ceremony(jar);
    let Some(Ceremony::Register(state)) = ceremony else {
        return Err(ApiError::BadRequest(
            "no passkey registration in progress".into(),
        ));
    };
    // The setup token only ever creates the first passkey, even if two setups race.
    if !was_signed_in && !passkeys(&s.db).await?.is_empty() {
        return Err(ApiError::Forbidden(
            "a passkey already exists: sign in to add another".into(),
        ));
    }
    let passkey = s
        .auth
        .webauthn
        .finish_passkey_registration(&credential, &state)?;
    save_passkey(&s.db, &passkey).await?;
    let jar = sign_in(&s.auth, jar)?;
    Ok((
        jar,
        Json(AuthStatus {
            signed_in: true,
            needs_setup: false,
        }),
    ))
}

async fn sign_in_start(
    State(s): State<AppState>,
    jar: PrivateCookieJar,
) -> Result<(PrivateCookieJar, Json<RequestChallengeResponse>), ApiError> {
    let existing = passkeys(&s.db).await?;
    if existing.is_empty() {
        return Err(ApiError::BadRequest(
            "no passkey yet: use the setup link first".into(),
        ));
    }
    let (challenge, state) = s.auth.webauthn.start_passkey_authentication(&existing)?;
    Ok((
        start_ceremony(&s.auth, jar, Ceremony::SignIn(state))?,
        Json(challenge),
    ))
}

async fn sign_in_finish(
    State(s): State<AppState>,
    jar: PrivateCookieJar,
    Json(credential): Json<PublicKeyCredential>,
) -> Result<(PrivateCookieJar, Json<AuthStatus>), ApiError> {
    let (jar, ceremony) = take_ceremony(jar);
    let Some(Ceremony::SignIn(state)) = ceremony else {
        return Err(ApiError::BadRequest("no sign-in in progress".into()));
    };
    let result = s
        .auth
        .webauthn
        .finish_passkey_authentication(&credential, &state)?;
    for mut passkey in passkeys(&s.db).await? {
        if passkey.update_credential(&result).is_some() {
            update_passkey(&s.db, &passkey).await?;
        }
    }
    let jar = sign_in(&s.auth, jar)?;
    Ok((
        jar,
        Json(AuthStatus {
            signed_in: true,
            needs_setup: false,
        }),
    ))
}

async fn sign_out(jar: PrivateCookieJar) -> (PrivateCookieJar, Json<AuthStatus>) {
    (
        jar.remove(Cookie::build(SESSION_COOKIE).path("/")),
        Json(AuthStatus {
            signed_in: false,
            needs_setup: false,
        }),
    )
}

async fn owner(db: &SqlitePool) -> Result<Uuid, ApiError> {
    sqlx::query("INSERT OR IGNORE INTO owner (id, user_id) VALUES (1, ?)")
        .bind(Uuid::new_v4().to_string())
        .execute(db)
        .await?;
    let id: String = sqlx::query("SELECT user_id FROM owner WHERE id = 1")
        .fetch_one(db)
        .await?
        .try_get("user_id")?;
    Uuid::parse_str(&id).map_err(|e| ApiError::BadRequest(format!("stored owner id: {e}")))
}

async fn passkeys(db: &SqlitePool) -> Result<Vec<Passkey>, ApiError> {
    let rows = sqlx::query("SELECT passkey FROM passkeys ORDER BY id")
        .fetch_all(db)
        .await?;
    rows.iter()
        .map(|r| Ok(serde_json::from_str(r.try_get("passkey")?)?))
        .collect()
}

async fn save_passkey(db: &SqlitePool, passkey: &Passkey) -> Result<(), ApiError> {
    sqlx::query("INSERT INTO passkeys (cred_id, passkey, created_at) VALUES (?, ?, ?)")
        .bind(serde_json::to_string(passkey.cred_id())?)
        .bind(serde_json::to_string(passkey)?)
        .bind(Timestamp::now().as_millisecond())
        .execute(db)
        .await?;
    Ok(())
}

async fn update_passkey(db: &SqlitePool, passkey: &Passkey) -> Result<(), ApiError> {
    sqlx::query("UPDATE passkeys SET passkey = ?, last_used_at = ? WHERE cred_id = ?")
        .bind(serde_json::to_string(passkey)?)
        .bind(Timestamp::now().as_millisecond())
        .bind(serde_json::to_string(passkey.cred_id())?)
        .execute(db)
        .await?;
    Ok(())
}
