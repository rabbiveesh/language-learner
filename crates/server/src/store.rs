//! SQLite access. Runtime-checked queries for now (see the kitten death toll in the PR):
//! switch to `sqlx::query!` with an offline `.sqlx` cache once the schema settles.

use domain::{Card, CardId, CardKind, Grade, Language, Note, NoteId, NoteKind, Review, Signals};
use jiff::Timestamp;
use sqlx::{Row, SqlitePool};

use crate::error::ApiError;

fn millis(t: Timestamp) -> i64 {
    t.as_millisecond()
}

fn from_millis(ms: i64) -> Result<Timestamp, ApiError> {
    Timestamp::from_millisecond(ms).map_err(|e| ApiError::BadRequest(e.to_string()))
}

pub async fn insert_note(
    db: &SqlitePool,
    language: &Language,
    kind: &NoteKind,
    now: Timestamp,
) -> Result<NoteId, ApiError> {
    let id = sqlx::query("INSERT INTO notes (language, kind, created_at) VALUES (?, ?, ?)")
        .bind(&language.0)
        .bind(serde_json::to_string(kind)?)
        .bind(millis(now))
        .execute(db)
        .await?
        .last_insert_rowid();
    Ok(NoteId(id))
}

pub async fn insert_card(
    db: &SqlitePool,
    note: NoteId,
    kind: CardKind,
    now: Timestamp,
) -> Result<CardId, ApiError> {
    let id = sqlx::query("INSERT INTO cards (note_id, kind, created_at) VALUES (?, ?, ?)")
        .bind(note.0)
        .bind(serde_json::to_string(&kind)?)
        .bind(millis(now))
        .execute(db)
        .await?
        .last_insert_rowid();
    Ok(CardId(id))
}

pub async fn card_exists(db: &SqlitePool, card: CardId) -> Result<bool, ApiError> {
    Ok(sqlx::query("SELECT 1 FROM cards WHERE id = ?")
        .bind(card.0)
        .fetch_optional(db)
        .await?
        .is_some())
}

pub async fn insert_review(db: &SqlitePool, review: &Review) -> Result<(), ApiError> {
    sqlx::query("INSERT INTO reviews (card_id, at, grade, signals) VALUES (?, ?, ?, ?)")
        .bind(review.card_id.0)
        .bind(millis(review.at))
        .bind(serde_json::to_string(&review.grade)?)
        .bind(serde_json::to_string(&review.signals)?)
        .execute(db)
        .await?;
    Ok(())
}

/// Every card with its note, oldest first.
pub async fn cards_with_notes(db: &SqlitePool) -> Result<Vec<(Card, Note)>, ApiError> {
    let rows = sqlx::query(
        "SELECT c.id, c.kind AS card_kind, n.id AS note_id, n.language, n.kind AS note_kind
         FROM cards c JOIN notes n ON n.id = c.note_id ORDER BY c.id",
    )
    .fetch_all(db)
    .await?;
    rows.iter()
        .map(|r| {
            let note_id = NoteId(r.try_get("note_id")?);
            let card = Card {
                id: CardId(r.try_get("id")?),
                note_id,
                kind: serde_json::from_str(r.try_get("card_kind")?)?,
            };
            let note = Note {
                id: note_id,
                language: Language(r.try_get("language")?),
                kind: serde_json::from_str(r.try_get("note_kind")?)?,
            };
            Ok((card, note))
        })
        .collect()
}

/// One card's reviews, oldest first.
pub async fn reviews_for(db: &SqlitePool, card: CardId) -> Result<Vec<Review>, ApiError> {
    let rows =
        sqlx::query("SELECT at, grade, signals FROM reviews WHERE card_id = ? ORDER BY at, id")
            .bind(card.0)
            .fetch_all(db)
            .await?;
    rows.iter()
        .map(|r| {
            let grade: Grade = serde_json::from_str(r.try_get("grade")?)?;
            let signals: Signals = serde_json::from_str(r.try_get("signals")?)?;
            Ok(Review {
                card_id: card,
                at: from_millis(r.try_get("at")?)?,
                grade,
                signals,
            })
        })
        .collect()
}
