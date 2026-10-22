# Fala

Fala ("speak" in Portuguese) is a personal app for learning languages the Fluent Forever way: pictures instead of translations,
native-speaker audio, and spaced repetition built in. Brazilian Portuguese first, Japanese next.

- Plan and decisions: [Language learner rewrite](https://claude.ai/artifact/JJQAhU3bnLwbUnKX4UyDeU)
- How to work on it: [CLAUDE.md](CLAUDE.md)

## Layout

| Path | What |
| --- | --- |
| `crates/domain` | Notes, cards, review log, typed-answer grading, FSRS scheduling |
| `crates/adapt` | Adaptive difficulty engine (generic, from the games) |
| `crates/media` | Picture / audio / dictionary sources and the modesty filter |
| `crates/eval` | Scores scheduler variants on a real review history (an Anki export) |
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

Use npm 11 or later (Node 24); npm 10 trips over a peer dependency in jsdom.

Settings come from the environment:

| Variable | Default | What |
| --- | --- | --- |
| `DATABASE_URL` | `sqlite://data/fala.db` | The SQLite database |
| `PORT` | `8080` | Where the server listens |
| `WEB_DIST` | `web/dist` | The built web app |
| `PUBLIC_ORIGIN` | `http://localhost:8080` | Where the app is served; passkeys are tied to its host. Any localhost port works in dev |
| `SESSION_KEY` | random on localhost | Encrypts the sign-in cookie; at least 64 bytes (`openssl rand -hex 64`). Required outside localhost |
| `SETUP_TOKEN` | none | Lets the first passkey be created, at `/setup?token=…` |

Run locally with `SETUP_TOKEN=anything cargo run -p server`, then open
`http://localhost:8080/setup?token=anything` (or the Vite port) to create a passkey.

Deploying to Fly (fala.veesh.me): [docs/deploy.md](docs/deploy.md).
