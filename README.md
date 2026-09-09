# alpha-progression

An unofficial Rust client for the [Alpha Progression](https://alphaprogression.com)
fitness app's private API, reverse-engineered from traffic captures. Builds for
native targets and for `wasm32-unknown-unknown`, with a `wasm-bindgen` wrapper for
use from JavaScript.

Not affiliated with, endorsed by, or supported by Alpha Progression.

## What it does

Auth (register / login) goes through `alphaprogression.com/api/v3/`. Everything
else - workouts, measurements, settings - lives in a per-user CouchDB database on
IBM Cloudant that the app syncs with via PouchDB replication. This crate can talk to both.

```rust
let client = alpha_progression::AlphaProgressionClient::new();

let session = client.login("you@example.com", "hunter2").await?;
// Session { uuid: "<64 hex>", email: "you@example.com", token: "<64 hex>" }
// The token doubles as the Cloudant password, so this is all you need.

let info = client.db_info(&session).await?;
println!("{} documents", info.doc_count);
// DbInfo { db_name: "<uuid>", doc_count: 27, doc_del_count: 0,
//          update_seq: "28-g1AAAAB5eJzLYWBgYMxgTmFQS0lK..." }

// _changes lists every doc in the database: profile, settings, the
// measurement series, and one per workout. "0" starts from the beginning.
let changes = client.get_changes(&session, "0", 100).await?;
// ChangesResponse { results: [...], last_seq: "28-g1AAAAB5...", pending: 0 }
//
// Each result is a ChangeEntry { id, seq, changes: [RevInfo { rev }] }:
//   id: "profile"                 rev: "1-6f2c..."
//   id: "settings"                rev: "2-b29a..."
//   id: "measurement-bodyweight"  rev: "9-a04b..."
//   id: "1788882599948-iajevnjn"  rev: "24-bb3b..."   <- a workout

// Workout doc IDs are {unix_millis}-{random}, assigned by the app when the
// workout is created, so pick them out of the list rather than guess one.
let workouts = changes
    .results
    .iter()
    .filter(|c| matches!(c.id.split_once('-'), Some((ms, _)) if ms.parse::<u64>().is_ok()));

for entry in workouts {
    let workout = client.get_workout(&session, &entry.id).await?;
    println!("{}: {} exercises", workout.name, workout.exercises.len());
    // WorkoutDoc {
    //     id: "1788882599948-iajevnjn",
    //     rev: Some("24-bb3b..."),
    //     name: "Standalone workout",
    //     exercises: [Exercise {
    //         id: "1788882599945",
    //         exercise_id: "exercise-50",   // the app's own exercise catalogue
    //         reps_target: Some(10),
    //         sets: [ExerciseSet { reps: 10, weight: 12.5, weight_unit: "kg",
    //                              completed: 1788882608476 }],
    //     }],
    // }
}
```

`get_changes` takes a `since` sequence, so passing back `changes.last_seq` on the next
call returns only what changed since — the same thing the app does to sync.

## Setup

The app authenticates each endpoint with a hardcoded `apiKey`, and salts passwords
with a static app-wide value before hashing. **Those values are not included in this
repository** — you have to extract them from the app yourself. There's a script that
does it in one command:
[docs/extracting-api-keys.md](docs/extracting-api-keys.md).

Once you have them:

```bash
cp .env.example .env
# fill in the values, then
cargo build -p alpha-progression
```

`build.rs` reads `.env` and bakes the values in at compile time, so they work on
`wasm32` too. Real environment variables take precedence, which is what you want in
CI. If a value is missing you get a compile error naming the variable.

## Building

```bash
# Native
cargo build -p alpha-progression

# WASM library
cargo build -p alpha-progression --target wasm32-unknown-unknown

# JS package — no-modules is required for file:// to work
wasm-pack build alpha-progression-wasm --target no-modules

# Self-contained HTML demo, no server needed
python3 bundle.py    # => example/index.bundle.html
```

The bundled HTML is gitignored: it inlines the compiled WASM, which has the keys and
salt baked into it. Build it locally rather than committing it.

See [docs/how-to-create-bundled-html.md](docs/how-to-create-bundled-html.md) for why
`--target web` doesn't work here, and [example/](example/) for a working browser client.

## Tests

Unit tests run offline. The integration tests hit the live API and are `#[ignore]`d by
default; they need `AP_TEST_EMAIL`, `AP_TEST_PASSWORD` and `AP_TEST_UUID` in `.env`.

```bash
cargo test -p alpha-progression
cargo test -p alpha-progression -- --ignored --nocapture
```

## Layout

| Path | Purpose |
| --- | --- |
| `alpha-progression/` | core library, native + wasm |
| `alpha-progression-wasm/` | `wasm-bindgen` wrapper |
| `example/` | browser client demo: `index.html` |
| `tools/extract-keys.mjs` | helper to get the apiKeys from the app bundle |
| `docs/extracting-api-keys.md` | how to get the keys and salt out of the app |
| `docs/alpha-progression-reference.md` | full API reference - routes, headers, bodies |

## Disclaimer

This is an interoperability and personal-data-portability project, published for
educational purposes. It talks to a private API that can change or break without
notice. Using it may conflict with Alpha Progression's terms of service - that's your
call to make. Use it with your own account and your own data. **No secrets** are shared
inside of the repository.

## AI disclosure

The library was written using Claude Code after the API was reverse engineered.
