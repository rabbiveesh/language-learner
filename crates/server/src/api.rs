//! The JSON API, mounted under `/api`.

use axum::extract::{Query, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use domain::{Card, CardId, CardState, Grade, Language, Note, NoteKind, Review, Signals, Word};
use jiff::Timestamp;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::AppState;
use crate::error::ApiError;
use crate::store;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/health", get(|| async { "ok" }))
        .route("/due", get(due))
        .route("/reviews", post(review))
        .route("/notes", post(new_word_note))
}

/// A card ready to study, with what it needs to render.
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export)]
pub struct DueCard {
    pub card: Card,
    pub note: Note,
    pub state: CardState,
}

#[derive(Debug, Deserialize)]
struct DueQuery {
    limit: Option<usize>,
}

/// New cards offered per call.
// TODO(design): new cards come from the adaptive engine (which words, which card kinds, how
// many today), not a fixed count. See the plan's "SRS" section.
const NEW_PER_CALL: usize = 10;

/// Due cards, most overdue first, then up to [`NEW_PER_CALL`] new ones.
async fn due(
    State(s): State<AppState>,
    Query(q): Query<DueQuery>,
) -> Result<Json<Vec<DueCard>>, ApiError> {
    let now = Timestamp::now();
    // TODO(design): cache CardState per card instead of replaying every log on each call. Fine
    // at one learner's scale for now.
    let mut due = Vec::new();
    let mut new = Vec::new();
    for (card, note) in store::cards_with_notes(&s.db).await? {
        let state = s
            .scheduler
            .replay(&store::reviews_for(&s.db, card.id).await?)?;
        match state.due {
            None => new.push(DueCard { card, note, state }),
            Some(d) if d <= now => due.push(DueCard { card, note, state }),
            Some(_) => {}
        }
    }
    due.sort_by_key(|c| c.state.due);
    due.extend(new.into_iter().take(NEW_PER_CALL));
    due.truncate(q.limit.unwrap_or(usize::MAX));
    Ok(Json(due))
}

#[derive(Debug, Clone, Deserialize, TS)]
#[ts(export)]
pub struct NewReview {
    pub card_id: CardId,
    pub grade: Grade,
    #[serde(default)]
    pub signals: Signals,
}

/// Log a review (the server stamps the time) and return the card's new state.
async fn review(
    State(s): State<AppState>,
    Json(r): Json<NewReview>,
) -> Result<Json<CardState>, ApiError> {
    if !store::card_exists(&s.db, r.card_id).await? {
        return Err(ApiError::NotFound);
    }
    let review = Review {
        card_id: r.card_id,
        at: Timestamp::now(),
        grade: r.grade,
        signals: r.signals,
    };
    store::insert_review(&s.db, &review).await?;
    Ok(Json(
        s.scheduler
            .replay(&store::reviews_for(&s.db, r.card_id).await?)?,
    ))
}

#[derive(Debug, Clone, Deserialize, TS)]
#[ts(export)]
pub struct NewWordNote {
    pub language: Language,
    pub word: Word,
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export)]
pub struct NoteWithCards {
    pub note: Note,
    pub cards: Vec<Card>,
}

/// Add a word and the cards its media allows.
// TODO(design): the "add a word" flow fills in IPA, gloss, picture and audio from the media
// sources (with the modesty filter) before saving; for now the client sends a finished word.
async fn new_word_note(
    State(s): State<AppState>,
    Json(n): Json<NewWordNote>,
) -> Result<Json<NoteWithCards>, ApiError> {
    if n.word.spelling.trim().is_empty() {
        return Err(ApiError::BadRequest("a word needs a spelling".into()));
    }
    let now = Timestamp::now();
    let kind = NoteKind::Word(n.word);
    let id = store::insert_note(&s.db, &n.language, &kind, now).await?;
    let note = Note {
        id,
        language: n.language,
        kind,
    };
    let mut cards = Vec::new();
    for kind in domain::card::kinds_for(&note) {
        cards.push(Card {
            id: store::insert_card(&s.db, id, kind, now).await?,
            note_id: id,
            kind,
        });
    }
    Ok(Json(NoteWithCards { note, cards }))
}
