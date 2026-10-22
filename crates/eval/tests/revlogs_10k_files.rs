//! Reading one user's file from the anki-revlogs-10k dataset, built by hand in its layout.

use std::path::Path;
use std::sync::Arc;

use eval::revlogs_10k::load;
use eval::{LearningDay, ReviewKind, samples};
use parquet::basic::Compression;
use parquet::data_type::Int64Type;
use parquet::file::properties::WriterProperties;
use parquet::file::writer::SerializedFileWriter;
use parquet::schema::parser::parse_message_type;

/// One row: card, day, rating, state (0 first learn, 1 learning step, 2 review, 3 relearning,
/// 4 a filtered-deck review that rescheduled).
type Row = (i64, i64, i64, i64);

/// The dataset's own schema: every column is an int64.
const SCHEMA: &str = "message schema {
    optional int64 card_id;
    optional int64 day_offset;
    optional int64 rating;
    optional int64 state;
    optional int64 duration;
    optional int64 elapsed_days;
    optional int64 elapsed_seconds;
}";

fn write(path: &Path, schema: &str, rows: &[Row]) {
    let schema = Arc::new(parse_message_type(schema).unwrap());
    let mut names = schema
        .get_fields()
        .iter()
        .map(|f| f.name().to_owned())
        .collect::<Vec<_>>()
        .into_iter();
    let props = Arc::new(
        WriterProperties::builder()
            .set_compression(Compression::SNAPPY)
            .build(),
    );
    let mut writer =
        SerializedFileWriter::new(std::fs::File::create(path).unwrap(), schema, props).unwrap();
    let mut group = writer.next_row_group().unwrap();
    while let Some(mut column) = group.next_column().unwrap() {
        let pick: fn(&Row) -> i64 = match names.next().unwrap().as_str() {
            "card_id" => |r| r.0,
            "day_offset" => |r| r.1,
            "rating" => |r| r.2,
            "state" => |r| r.3,
            "duration" => |_| 4000,
            _ => |_| -1,
        };
        let values: Vec<i64> = rows.iter().map(pick).collect();
        let defined = vec![1i16; rows.len()];
        column
            .typed::<Int64Type>()
            .write_batch(&values, Some(&defined), None)
            .unwrap();
        column.close().unwrap();
    }
    group.close().unwrap();
    writer.close().unwrap();
}

fn written(rows: &[Row]) -> (tempfile::TempDir, std::path::PathBuf) {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("data.parquet");
    write(&file, SCHEMA, rows);
    (dir, file)
}

#[test]
fn a_users_log_reads_in_review_order_with_each_cards_state_as_its_kind() {
    let (_dir, file) = written(&[
        (7, 0, 1, 0), // card 7 seen for the first time: forgot
        (7, 0, 3, 1), // learning step, same day
        (8, 0, 3, 0),
        (7, 2, 3, 2),
        (8, 5, 1, 2), // lapse
        (8, 6, 3, 3), // relearned the next day
        (7, 9, 3, 4), // reviewed early from a filtered deck
    ]);

    let log = load(&file).unwrap();
    let kinds: Vec<(i64, ReviewKind)> = log.iter().map(|r| (r.card_id, r.kind)).collect();
    assert_eq!(
        kinds,
        [
            (7, ReviewKind::Learning),
            (7, ReviewKind::Learning),
            (8, ReviewKind::Learning),
            (7, ReviewKind::Review),
            (8, ReviewKind::Review),
            (8, ReviewKind::Relearning),
            (7, ReviewKind::Review),
        ]
    );
    assert_eq!(log[3].day, LearningDay(2));
    assert_eq!(log[0].duration_ms, 4000);

    let targets = samples(&log);
    let order: Vec<(i64, bool)> = targets.iter().map(|s| (s.card_id, s.recalled())).collect();
    assert_eq!(
        order,
        [(7, true), (8, false), (8, true), (7, true)],
        "one target per later day, in order"
    );
    assert_eq!(targets[0].new_that_day, 2);
}

#[test]
fn a_card_reset_to_new_starts_its_history_over() {
    let (_dir, file) = written(&[(1, 0, 3, 0), (1, 3, 1, 2), (1, 4, 3, 0), (1, 7, 3, 2)]);
    let targets = samples(&load(&file).unwrap());
    assert_eq!(targets.len(), 1);
    assert_eq!(targets[0].item.reviews.len(), 2);
}

#[test]
fn a_parquet_file_that_isnt_a_revlog_names_the_missing_column() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("cards.parquet");
    write(
        &file,
        "message schema { optional int64 card_id; optional int64 rating; }",
        &[(1, 0, 3, 0)],
    );
    let err = load(&file).unwrap_err();
    assert!(err.to_string().contains("day_offset"), "{err}");
}

#[test]
fn a_log_out_of_review_order_is_refused_rather_than_misread() {
    let (_dir, file) = written(&[(1, 4, 3, 0), (2, 1, 3, 0)]);
    let err = load(&file).unwrap_err();
    assert!(err.to_string().contains("review order"), "{err}");
}

#[test]
fn a_state_the_dataset_never_writes_is_refused() {
    let (_dir, file) = written(&[(1, 0, 3, 0), (1, 2, 3, 7)]);
    let err = load(&file).unwrap_err();
    assert!(err.to_string().contains("state"), "{err}");
}
