//! Reading an Anki collection: a `.colpkg` / `.apkg` export, or a bare `collection.anki2[1]`.
//!
//! Exports are zip files. Newer ones hold `collection.anki21b` (SQLite, zstd-compressed) next to
//! a stub `collection.anki2` that only says "please update"; older ones hold `collection.anki21`
//! or `collection.anki2` directly.

use std::collections::HashMap;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use jiff::Timestamp;
use sqlx::sqlite::SqliteConnectOptions;
use sqlx::{Connection, Row, SqliteConnection};

use crate::history::{LoggedReview, ReviewKind};

#[derive(Debug, thiserror::Error)]
pub enum AnkiError {
    #[error("can't read {path}: {source}")]
    Io {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("not an Anki export: {0}")]
    Zip(#[from] zip::result::ZipError),
    #[error("the export has no collection inside")]
    NoCollection,
    #[error("reading the collection: {0}")]
    Db(#[from] sqlx::Error),
    #[error("the collection's deck list is unreadable: {0}")]
    Decks(#[from] serde_json::Error),
}

/// The review log and which deck each card is in.
#[derive(Debug, Clone, Default)]
pub struct Collection {
    pub reviews: Vec<LoggedReview>,
    /// Card id → full deck name, with `::` between levels.
    pub decks: HashMap<i64, String>,
}

impl Collection {
    /// Only the reviews of cards whose deck name contains `filter` (case-insensitive).
    pub fn in_deck(&self, filter: &str) -> Vec<LoggedReview> {
        let filter = filter.to_lowercase();
        self.reviews
            .iter()
            .filter(|r| {
                self.decks
                    .get(&r.card_id)
                    .is_some_and(|d| d.to_lowercase().contains(&filter))
            })
            .cloned()
            .collect()
    }
}

/// Read a collection from an export or a bare collection file.
pub async fn load(path: &Path) -> Result<Collection, AnkiError> {
    let io = |source| AnkiError::Io {
        path: path.to_owned(),
        source,
    };
    let mut file = std::fs::File::open(path).map_err(io)?;
    let mut magic = [0u8; 4];
    file.read_exact(&mut magic).map_err(io)?;
    if &magic != b"PK\x03\x04" {
        return read_db(path).await;
    }
    let db = unpack(std::fs::File::open(path).map_err(io)?)?;
    read_db(db.path()).await
}

/// Copy the newest collection format in the zip out to a temp file.
fn unpack(file: std::fs::File) -> Result<tempfile::NamedTempFile, AnkiError> {
    let mut zip = zip::ZipArchive::new(file)?;
    let tmp_err = |source| AnkiError::Io {
        path: std::env::temp_dir(),
        source,
    };
    let mut out = tempfile::NamedTempFile::new().map_err(tmp_err)?;
    for (name, compressed) in [
        ("collection.anki21b", true),
        ("collection.anki21", false),
        ("collection.anki2", false),
    ] {
        let Ok(entry) = zip.by_name(name) else {
            continue;
        };
        if compressed {
            zstd::stream::copy_decode(entry, &mut out).map_err(tmp_err)?;
        } else {
            let mut entry = entry;
            std::io::copy(&mut entry, &mut out).map_err(tmp_err)?;
        }
        out.flush().map_err(tmp_err)?;
        return Ok(out);
    }
    Err(AnkiError::NoCollection)
}

async fn read_db(path: &Path) -> Result<Collection, AnkiError> {
    let mut db =
        SqliteConnection::connect_with(&SqliteConnectOptions::new().filename(path).read_only(true))
            .await?;
    let rows = sqlx::query("SELECT id, cid, ease, type, factor, time FROM revlog ORDER BY id")
        .fetch_all(&mut db)
        .await?;
    let mut reviews = Vec::with_capacity(rows.len());
    for r in rows {
        let id: i64 = r.try_get("id")?;
        let ease: i64 = r.try_get("ease")?;
        let factor: i64 = r.try_get("factor")?;
        let kind = match r.try_get::<i64, _>("type")? {
            0 => ReviewKind::Learning,
            1 => ReviewKind::Review,
            2 => ReviewKind::Relearning,
            3 if factor == 0 => ReviewKind::Filtered,
            3 => ReviewKind::Review,
            _ => ReviewKind::Manual,
        };
        let Ok(at) = Timestamp::from_millisecond(id) else {
            continue;
        };
        reviews.push(LoggedReview {
            card_id: r.try_get("cid")?,
            at,
            rating: ease.clamp(0, 4) as u8,
            kind,
            duration_ms: r.try_get::<i64, _>("time")?.clamp(0, i64::from(u32::MAX)) as u32,
        });
    }
    let decks = card_decks(&mut db).await?;
    Ok(Collection { reviews, decks })
}

/// Card id → deck name. Newer collections have a `decks` table; older ones keep the decks as
/// JSON in `col.decks`. A card in a filtered deck belongs to its original deck (`odid`).
async fn card_decks(db: &mut SqliteConnection) -> Result<HashMap<i64, String>, AnkiError> {
    let names: HashMap<i64, String> = match sqlx::query("SELECT id, name FROM decks")
        .fetch_all(&mut *db)
        .await
    {
        Ok(rows) => rows
            .iter()
            .map(|r| {
                Ok((
                    r.try_get::<i64, _>("id")?,
                    r.try_get::<String, _>("name")?.replace('\x1f', "::"),
                ))
            })
            .collect::<Result<_, sqlx::Error>>()?,
        Err(_) => {
            let json: String = sqlx::query("SELECT decks FROM col")
                .fetch_one(&mut *db)
                .await?
                .try_get("decks")?;
            let decks: HashMap<String, serde_json::Value> = serde_json::from_str(&json)?;
            decks
                .into_iter()
                .filter_map(|(id, d)| Some((id.parse().ok()?, d.get("name")?.as_str()?.to_owned())))
                .collect()
        }
    };
    let rows =
        sqlx::query("SELECT id, CASE WHEN odid != 0 THEN odid ELSE did END AS deck FROM cards")
            .fetch_all(&mut *db)
            .await?;
    let mut out = HashMap::with_capacity(rows.len());
    for r in rows {
        if let Some(name) = names.get(&r.try_get::<i64, _>("deck")?) {
            out.insert(r.try_get("id")?, name.clone());
        }
    }
    Ok(out)
}
