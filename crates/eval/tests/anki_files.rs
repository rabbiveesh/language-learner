//! Reading real Anki file layouts, built by hand: a current export and an old one.

use std::io::Write;
use std::path::Path;

use eval::anki;
use eval::{DayClock, ReviewKind};
use jiff::tz::TimeZone;
use sqlx::sqlite::SqliteConnectOptions;
use sqlx::{Connection, SqliteConnection};
use zip::write::SimpleFileOptions;

async fn load(path: &Path) -> Result<anki::Collection, anki::AnkiError> {
    let clock = DayClock {
        tz: TimeZone::UTC,
        rollover_hour: 4,
    };
    anki::load(path, &clock).await
}

const REVLOG: &str =
    "CREATE TABLE revlog (id INTEGER PRIMARY KEY, cid INTEGER, usn INTEGER, ease INTEGER,
    ivl INTEGER, lastIvl INTEGER, factor INTEGER, time INTEGER, type INTEGER)";

/// Two cards: one in a Portuguese deck, one in a Japanese deck (sitting in a filtered deck).
async fn collection(path: &Path, modern: bool) {
    let opts = SqliteConnectOptions::new()
        .filename(path)
        .create_if_missing(true);
    let mut db = SqliteConnection::connect_with(&opts).await.unwrap();
    let mut stmts = vec![
        REVLOG.to_string(),
        "CREATE TABLE cards (id INTEGER PRIMARY KEY, did INTEGER, odid INTEGER)".into(),
        "INSERT INTO cards VALUES (100, 1, 0), (200, 9, 2)".into(),
        // card 100: learn, review; card 200: learn, manual reschedule, cram without rescheduling
        "INSERT INTO revlog VALUES
            (1700000000000, 100, 0, 3, -600, 0, 0, 5000, 0),
            (1700259200000, 100, 0, 3, 7, 3, 2500, 4000, 1),
            (1700000100000, 200, 0, 1, -60, 0, 0, 9000, 0),
            (1700100000000, 200, 0, 0, 5, 0, 0, 0, 4),
            (1700200000000, 200, 0, 3, 0, 0, 0, 3000, 3)"
            .into(),
    ];
    if modern {
        stmts.push("CREATE TABLE decks (id INTEGER PRIMARY KEY, name TEXT)".into());
        stmts.push("INSERT INTO decks VALUES (1, 'Languages\x1fPortuguese'), (2, 'Languages\x1fJapanese'), (9, 'Cram')".into());
    } else {
        stmts.push("CREATE TABLE col (decks TEXT)".into());
        stmts.push(
            r#"INSERT INTO col VALUES ('{"1": {"name": "Languages::Portuguese"}, "2": {"name": "Languages::Japanese"}, "9": {"name": "Cram"}}')"#
                .into(),
        );
    }
    for s in stmts {
        sqlx::query(sqlx::AssertSqlSafe(s))
            .execute(&mut db)
            .await
            .unwrap();
    }
    db.close().await.unwrap();
}

fn zip_up(out: &Path, entries: &[(&str, Vec<u8>)]) {
    let mut zip = zip::ZipWriter::new(std::fs::File::create(out).unwrap());
    for (name, bytes) in entries {
        zip.start_file(*name, SimpleFileOptions::default()).unwrap();
        zip.write_all(bytes).unwrap();
    }
    zip.finish().unwrap();
}

#[tokio::test]
async fn a_current_colpkg_export_is_read_from_its_compressed_collection() {
    let dir = tempfile::tempdir().unwrap();
    let db = dir.path().join("real.anki21");
    collection(&db, true).await;
    let compressed = zstd::encode_all(std::fs::read(&db).unwrap().as_slice(), 0).unwrap();
    let export = dir.path().join("collection.colpkg");
    zip_up(
        &export,
        &[
            ("collection.anki2", b"please update".to_vec()),
            ("collection.anki21b", compressed),
        ],
    );

    let c = load(&export).await.unwrap();
    assert_eq!(c.reviews.len(), 5);
    assert_eq!(c.decks[&100], "Languages::Portuguese");
    assert_eq!(
        c.decks[&200], "Languages::Japanese",
        "a card in a filtered deck keeps its home deck"
    );
    assert_eq!(c.in_deck("japanese").len(), 3);

    let kinds: Vec<ReviewKind> = c.in_deck("japanese").iter().map(|r| r.kind).collect();
    assert_eq!(
        kinds,
        [
            ReviewKind::Learning,
            ReviewKind::Manual,
            ReviewKind::Filtered
        ]
    );
}

#[tokio::test]
async fn an_old_apkg_with_decks_as_json_is_read_too() {
    let dir = tempfile::tempdir().unwrap();
    let db = dir.path().join("old.anki2");
    collection(&db, false).await;
    let export = dir.path().join("deck.apkg");
    zip_up(
        &export,
        &[("collection.anki2", std::fs::read(&db).unwrap())],
    );

    let c = load(&export).await.unwrap();
    assert_eq!(c.in_deck("portuguese").len(), 2);
    assert_eq!(c.in_deck("portuguese")[1].rating, 3);
}

#[tokio::test]
async fn a_bare_collection_file_works_without_unzipping() {
    let dir = tempfile::tempdir().unwrap();
    let db = dir.path().join("collection.anki21");
    collection(&db, true).await;
    assert_eq!(load(&db).await.unwrap().reviews.len(), 5);
}

#[tokio::test]
async fn a_zip_without_a_collection_says_so() {
    let dir = tempfile::tempdir().unwrap();
    let export = dir.path().join("photos.zip");
    zip_up(&export, &[("cat.jpg", vec![1, 2, 3])]);
    let err = load(&export).await.unwrap_err();
    assert!(err.to_string().contains("no collection"), "{err}");
}
