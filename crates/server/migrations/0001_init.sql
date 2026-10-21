-- Notes: what the learner studies. `kind` is domain::NoteKind as JSON.
CREATE TABLE notes (
    id INTEGER PRIMARY KEY,
    language TEXT NOT NULL,
    kind TEXT NOT NULL,
    created_at INTEGER NOT NULL -- unix milliseconds
);

-- Cards: generated from notes. `kind` is domain::CardKind as JSON.
CREATE TABLE cards (
    id INTEGER PRIMARY KEY,
    note_id INTEGER NOT NULL REFERENCES notes(id) ON DELETE CASCADE,
    kind TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    UNIQUE (note_id, kind)
);

-- The review log: append-only, the source of truth for scheduling.
CREATE TABLE reviews (
    id INTEGER PRIMARY KEY,
    card_id INTEGER NOT NULL REFERENCES cards(id),
    at INTEGER NOT NULL, -- unix milliseconds
    grade TEXT NOT NULL, -- domain::Grade as JSON
    signals TEXT NOT NULL -- domain::Signals as JSON
);
CREATE INDEX reviews_by_card ON reviews (card_id, at);

CREATE TRIGGER reviews_no_update BEFORE UPDATE ON reviews
BEGIN SELECT RAISE(ABORT, 'the review log is append-only'); END;
CREATE TRIGGER reviews_no_delete BEFORE DELETE ON reviews
BEGIN SELECT RAISE(ABORT, 'the review log is append-only'); END;
