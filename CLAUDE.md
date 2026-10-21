# Language Learner

A personal Fluent Forever–style language app: words with pictures and native audio, studied with a
built-in spaced-repetition scheduler (FSRS) and an adaptive engine that keeps practice at the right
difficulty. Brazilian Portuguese first, Japanese second. One learner, one small server.

The plan (decisions, sources, milestones) is the "Language learner rewrite" doc linked from the
README. Anything marked `TODO(design)` in the code is a hole on purpose: it waits on a design
decision, so don't fill it with a guess.

## Golden rule: never reinvent wheels

Kittens die when you do that. Before writing any non-trivial utility, look for a well-maintained
library that already does it and use that (FSRS is the `fsrs` crate, time is `jiff`, string
distance is `strsim`). Hand-rolled code is only for logic that is genuinely specific to this app.

## Do things the right way, not the short way

The short way leads to more dead kittens. Prefer the properly typed, tested, maintainable approach
even when a shortcut would work today. TypeScript types are generated from the Rust structs with
ts-rs (`cargo test` writes them to `web/src/generated/`), never hand-copied or edited. CI runs the
tests and lints.

## Stack

- `crates/domain`: notes, cards, the review log, grading, scheduling. Pure: no I/O.
- `crates/adapt`: the adaptive difficulty engine, generic over the app's skills. Pure reducers.
- `crates/media`: where pictures, audio and dictionary data come from, behind traits, and the
  modesty filter.
- `crates/server`: axum + SQLite (sqlx). The only place with I/O, secrets and API keys.
- `web/`: Vue 3 + TypeScript, Vite, Nuxt UI (Tailwind v4), Pinia, vue-router, Vitest.
- Hosting: Fly, CPU only. Anything that needs a GPU (training the phoneme model) runs on Veesh's
  machine, and only the weights ship.
- Claude API: Veesh's plan includes monthly API credits, so prefer Anthropic models (Haiku-class)
  for glosses, picture checks, grading free sentences and generating grammar sentences. Everything
  else should be free or near free.

## Product rules

- **Pictures are tznius.** Every searched picture passes the modesty filter
  (`config/modesty.md`) before it's shown, and the filter fails closed: a picture that couldn't be
  checked is not shown. When nothing passes, suggest the learner upload their own picture.
- **The review log is append-only** and is the source of truth. Schedules are replayed from it.
- **Adaptation is invisible.** Bands, assists and calibration are never shown on screen.
- **No time pressure.** Response time is recorded as a signal and never shown; slow but right is
  right.
- **Fail gracefully.** A provider that's down means fewer results, never a broken page.
- **Recognition, typing and speaking are different skills**, so each card kind gets its own
  schedule parameters and its own adaptive skill.

## Assume you're wrong

When you confidently assume you're right, you kill kittens. Assume you're wrong about the
requirements, about how the learner uses the app, and about the data model. Shape the code so that
being wrong has the smallest possible blast radius:

- Keep modules small, with narrow interfaces, so one wrong idea is fixed in one place.
- Migrate stored data explicitly (`crates/server/migrations`), because the data model will change.
- Keep the UI decoupled from the learning rules and from storage. Rules live in Rust, never in Vue
  components.
- Prefer configuration and data over hard-coded behaviour (word lists, the modesty standard, tuning).
- Randomness is seeded and passed in, so every decision can be replayed in a test.

## Abstractions follow the data, and compose cleanly

Abstractions that aren't about the data make programming harder, and they kill kittens. So do
things that don't compose cleanly. Name and shape modules after the domain data (notes, cards,
reviews, skills), not after generic machinery. Prefer small pure functions and reducers that take
data and return data over managers, wrappers or class hierarchies.

Interfaces must not lie. A name, type or function must do exactly what it claims, no more and no
less.

## Tests express intent

Test names say what the learner would notice ("remembering a card takes it off today's list"), not
which function was called. The adapt crate also has simulated learners (`crates/adapt/tests`):
re-run them when you change a rule or a tuning value.

## Kitten death toll

Finish every piece of work (each PR, each reply that reports finished work) with a **kitten death
toll**: an honest count of the shortcuts taken and wheels reinvented, with each one named, why it
happened, and what would fix it. Zero is the goal. Never round down.
