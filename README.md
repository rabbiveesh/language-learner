# Language Learner

A personal app for learning languages the Fluent Forever way: pictures instead of translations,
native-speaker audio, and spaced repetition built in. Brazilian Portuguese first, Japanese next.

- Plan and decisions: [Language learner rewrite](https://claude.ai/artifact/JJQAhU3bnLwbUnKX4UyDeU)
- How to work on it: [CLAUDE.md](CLAUDE.md)

## Layout

| Path | What |
| --- | --- |
| `crates/domain` | Notes, cards, review log, typed-answer grading, FSRS scheduling |
| `crates/adapt` | Adaptive difficulty engine (generic, from the games) |
| `crates/media` | Picture / audio / dictionary sources and the modesty filter |
| `crates/eval` | Scores scheduler variants on a real review history (an Anki export, or a user from the FSRS dataset) |
| `crates/server` | axum + SQLite API, serves the built web app |
| `web/` | Vue 3 + Nuxt UI front end |
| `config/` | Plain-text configuration (the picture standard) |

## Running it

```sh
cargo test                      # all Rust tests; also regenerates web/src/generated
cargo run -p server             # API on :8080, database in data/

cd web
npm ci
npm run dev                     # Vite on :5173, proxies /api to :8080
npm run test:unit -- --run
```

To score scheduling ideas on your own history:

```sh
cargo run --release -p eval -- collection.colpkg --deck portuguese
```

Or on other people's: one user's `revlogs/user_id=N/data.parquet` from the FSRS project's
[anki-revlogs-10k](https://huggingface.co/datasets/open-spaced-repetition/anki-revlogs-10k)
dataset (gated: accept its terms, then download with a Hugging Face token).

```sh
cargo run --release -p eval -- revlogs/user_id=1/data.parquet
```

Use npm 11 or later (Node 24); npm 10 trips over a peer dependency in jsdom.

Settings come from the environment: `DATABASE_URL` (default `sqlite://data/language-learner.db`),
`PORT` (default 8080) and `WEB_DIST` (default `web/dist`).
