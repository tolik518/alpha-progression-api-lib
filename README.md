# alpha-progression

An unofficial Rust client for the [Alpha Progression](https://alphaprogression.com)
fitness app's private API, reverse-engineered from traffic captures. Builds for
native targets and for `wasm32-unknown-unknown`, with a `wasm-bindgen` wrapper for
use from JavaScript.

Not affiliated with, endorsed by, or supported by Alpha Progression.

## What it does

Auth (register / login) goes through `alphaprogression.com/api/v3/`. Everything
else — workouts, measurements, settings — lives in a per-user CouchDB database on
IBM Cloudant that the app syncs with via PouchDB replication. This crate wraps both.

```rust
let client = alpha_progression::AlphaProgressionClient::new();
let session = client.login("you@example.com", "hunter2").await?;

let info = client.db_info(&session).await?;
let workout = client.get_workout(&session, "1788882599948-iajevnjn").await?;
```

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
| `example/` | browser client — `index.html`, plus the generated standalone bundle |
| `tools/extract-keys.mjs` | resolves the apiKeys out of an app bundle |
| `docs/extracting-api-keys.md` | how to get the keys and salt out of the app |
| `docs/alpha-progression-reference.md` | full API reference — routes, headers, bodies |

## Disclaimer

This is an interoperability and personal-data-portability project, published for
educational purposes. It talks to a private API that can change or break without
notice. Using it may conflict with Alpha Progression's terms of service — that's your
call to make. Use it with your own account and your own data.
