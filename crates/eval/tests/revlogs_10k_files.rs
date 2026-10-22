//! Reading one user's file from the anki-revlogs-10k dataset, built by hand in its layout.

use std::path::Path;
use std::sync::Arc;

use eval::revlogs_10k::load;
use eval::{LearningDay, ReviewKind, samples};
use parquet::basic::Compression;
use parquet::data_type::{Int32Type, Int64Type};
use parquet::file::properties::WriterProperties;
use parquet::file::writer::SerializedFileWriter;
use parquet::schema::parser::parse_message_type;

/// One row: card, day, rating, duration in ms.
type Row = (i64, i32, i32, i32);

const SCHEMA: &str = "message schema {
    required int64 card_id;
    required int32 day_offset;
    required int32 rating;
    required int32 state;
    required int32 duration;
    required int32 elapsed_days;
    required int64 elapsed_seconds;
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
        let name = names.next().unwrap();
        let ints = |pick: fn(&Row) -> i32| rows.iter().map(pick).collect::<Vec<i32>>();
        match name.as_str() {
            "card_id" => {
                let cards: Vec<i64> = rows.iter().map(|r| r.0).collect();
                column.typed::<Int64Type>().write_batch(&cards, None, None)
            }
            "elapsed_seconds" => {
                let zeros = vec![0i64; rows.len()];
                column.typed::<Int64Type>().write_batch(&zeros, None, None)
            }
            "day_offset" => column
                .typed::<Int32Type>()
                .write_batch(&ints(|r| r.1), None, None),
            "rating" => column
                .typed::<Int32Type>()
                .write_batch(&ints(|r| r.2), None, None),
            "duration" => column
                .typed::<Int32Type>()
                .write_batch(&ints(|r| r.3), None, None),
            _ => column
                .typed::<Int32Type>()
                .write_batch(&ints(|_| 0), None, None),
        }
        .unwrap();
        column.close().unwrap();
    }
    group.close().unwrap();
    writer.close().unwrap();
}

#[test]
fn a_users_log_reads_in_review_order_with_each_cards_first_answer_as_its_learning_review() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("data.parquet");
    write(
        &file,
        SCHEMA,
        &[
            (7, 0, 0, 0),    // card 7 rescheduled by hand before it was ever studied
            (7, 0, 1, 9000), // learned: forgot
            (7, 0, 3, 4000), // learning step, same day
            (8, 0, 3, 5000),
            (7, 2, 3, 3000),
            (8, 5, 1, 6000),
        ],
    );

    let log = load(&file).unwrap();
    let kinds: Vec<(i64, ReviewKind)> = log.iter().map(|r| (r.card_id, r.kind)).collect();
    assert_eq!(
        kinds,
        [
            (7, ReviewKind::Manual),
            (7, ReviewKind::Learning),
            (7, ReviewKind::Review),
            (8, ReviewKind::Learning),
            (7, ReviewKind::Review),
            (8, ReviewKind::Review),
        ]
    );
    assert_eq!(log[4].day, LearningDay(2));
    assert_eq!(log[1].duration_ms, 9000);

    let targets = samples(&log);
    let order: Vec<(i64, bool)> = targets.iter().map(|s| (s.card_id, s.recalled())).collect();
    assert_eq!(
        order,
        [(7, true), (8, false)],
        "one target per later day, in order"
    );
    assert_eq!(targets[0].new_that_day, 2);
}

#[test]
fn a_parquet_file_that_isnt_a_revlog_names_the_missing_column() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("cards.parquet");
    write(
        &file,
        "message schema { required int64 card_id; required int32 rating; }",
        &[(1, 0, 3, 0)],
    );
    let err = load(&file).unwrap_err();
    assert!(err.to_string().contains("day_offset"), "{err}");
}

#[test]
fn a_log_out_of_review_order_is_refused_rather_than_misread() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("data.parquet");
    write(&file, SCHEMA, &[(1, 4, 3, 0), (2, 1, 3, 0)]);
    let err = load(&file).unwrap_err();
    assert!(err.to_string().contains("review order"), "{err}");
}
