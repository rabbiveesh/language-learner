//! Reading one user from the FSRS project's Anki Revlogs 10K dataset
//! (`open-spaced-repetition/anki-revlogs-10k` on Hugging Face): the file
//! `revlogs/user_id=N/data.parquet`.
//!
//! The dataset has no timestamps. Each row has the learning day it fell on (`day_offset`, days
//! since the collection started, already shifted by the user's time zone and rollover hour), and
//! the rows are in the order the reviews happened (the FSRS benchmark relies on both). That is
//! exactly a review log on learning days.
//!
//! The `state` column isn't read: its encoding is documented only on the gated dataset card, and
//! [`crate::samples`] needs no more than where each card's history starts. So a card's first row
//! is its [`ReviewKind::Learning`] review and every later answered row is a
//! [`ReviewKind::Review`], as the benchmark treats them.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use parquet::errors::ParquetError;
use parquet::file::reader::{FileReader, SerializedFileReader};
use parquet::record::Field;

use crate::history::{LearningDay, LoggedReview, ReviewKind};

#[derive(Debug, thiserror::Error)]
pub enum Revlogs10kError {
    #[error("can't read {path}: {source}")]
    Parquet { path: PathBuf, source: ParquetError },
    #[error("the file has no `{0}` column; is it a revlogs file from anki-revlogs-10k?")]
    MissingColumn(&'static str),
    #[error("row {row}: `{column}` isn't a whole number in range")]
    NotAnInteger { row: usize, column: &'static str },
    #[error(
        "row {row} is on an earlier day than the row before it; the log must be in review order"
    )]
    OutOfOrder { row: usize },
}

const CARD: &str = "card_id";
const DAY: &str = "day_offset";
const RATING: &str = "rating";
const DURATION: &str = "duration";

/// Read one user's review log, oldest first.
pub fn load(path: &Path) -> Result<Vec<LoggedReview>, Revlogs10kError> {
    let parquet = |source| Revlogs10kError::Parquet {
        path: path.to_owned(),
        source,
    };
    let reader = SerializedFileReader::try_from(path).map_err(parquet)?;
    let names: Vec<String> = reader
        .metadata()
        .file_metadata()
        .schema_descr()
        .root_schema()
        .get_fields()
        .iter()
        .map(|f| f.name().to_owned())
        .collect();
    let column = |name: &'static str| {
        names
            .iter()
            .position(|n| n == name)
            .ok_or(Revlogs10kError::MissingColumn(name))
    };
    let (card, day, rating, duration) = (
        column(CARD)?,
        column(DAY)?,
        column(RATING)?,
        column(DURATION)?,
    );

    let mut seen = HashSet::new();
    let mut reviews: Vec<LoggedReview> = Vec::new();
    for (row, fields) in reader.get_row_iter(None).map_err(parquet)?.enumerate() {
        let fields = fields.map_err(parquet)?.into_columns();
        let int = |i: usize, column: &'static str| {
            integer(&fields[i].1).ok_or(Revlogs10kError::NotAnInteger { row, column })
        };
        let card_id = int(card, CARD)?;
        let day = LearningDay(
            i32::try_from(int(day, DAY)?)
                .map_err(|_| Revlogs10kError::NotAnInteger { row, column: DAY })?,
        );
        if reviews.last().is_some_and(|prev| prev.day > day) {
            return Err(Revlogs10kError::OutOfOrder { row });
        }
        let rating = u8::try_from(int(rating, RATING)?)
            .ok()
            .filter(|r| *r <= 4)
            .ok_or(Revlogs10kError::NotAnInteger {
                row,
                column: RATING,
            })?;
        let kind = if rating == 0 {
            ReviewKind::Manual
        } else if seen.insert(card_id) {
            ReviewKind::Learning
        } else {
            ReviewKind::Review
        };
        reviews.push(LoggedReview {
            card_id,
            day,
            rating,
            kind,
            duration_ms: int(duration, DURATION)?.clamp(0, i64::from(u32::MAX)) as u32,
        });
    }
    Ok(reviews)
}

fn integer(field: &Field) -> Option<i64> {
    match *field {
        Field::Byte(v) => Some(v.into()),
        Field::Short(v) => Some(v.into()),
        Field::Int(v) => Some(v.into()),
        Field::Long(v) => Some(v),
        Field::UByte(v) => Some(v.into()),
        Field::UShort(v) => Some(v.into()),
        Field::UInt(v) => Some(v.into()),
        Field::ULong(v) => v.try_into().ok(),
        _ => None,
    }
}
