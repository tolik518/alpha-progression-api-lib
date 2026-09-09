# Alpha Progression API — Project Context

## What this is

A Rust library wrapping the **Alpha Progression** fitness app's private API,
reverse-engineered via HTTP Toolkit traffic capture (HAR files). Includes a
WASM build for use from JavaScript.

---

## Architecture

Two-crate workspace:

| Crate | Purpose |
|-------|---------|
| `crates/alpha-progression` | Core library — works on native and `wasm32-unknown-unknown` |
| `crates/alpha-progression-wasm` | Thin `wasm-bindgen` wrapper exposing the core to JavaScript |

The app (`com.alphaprogression.alphaprogression`) is a **Capacitor** (Ionic) app.
All business logic is in `assets/public/assets/index-xZcFl7Rl.js` inside the APK.

---

## API Overview

### Auth — `https://alphaprogression.com/api/v3/`

All endpoints use `application/x-www-form-urlencoded` POST bodies.

**Password hashing** (done client-side before sending):
```
SHA256(password + AP_PASSWORD_SALT)
```
Static pepper hardcoded in the JS bundle — same for all users.

**Endpoints and their `apiKey` values:**

Each endpoint carries its own static `apiKey`. The values are **not stored in this
repo** — they live in `.env` (gitignored) and are injected at compile time by
`alpha-progression/build.rs`. See `docs/extracting-api-keys.md`.

| Endpoint | Env var |
|----------|---------|
| `/user-signup` | `AP_API_KEY_SIGNUP` |
| `/user-login` | `AP_API_KEY_LOGIN` |
| `/track-month` | `AP_API_KEY_TRACK_MONTH` |
| `/check-subscription` | `AP_API_KEY_CHECK_SUBSCRIPTION` |
| `/set-currency` | `AP_API_KEY_SET_CURRENCY` |

Login returns `{ name: "<uuid>", token: "<session_token>" }`.
Register returns `{ token: "<session_token>" }` — UUID is client-generated (64 hex chars).

### Data — IBM Cloudant (CouchDB)

`https://8e4fc8f1-3681-4dc2-8215-45ad5f9233e6-bluemix.cloudant.com/{uuid}/`

Each user has their **own CouchDB database** named after their UUID.
Auth: `Authorization: Basic base64({uuid}:{session_token})` — credentials come directly from the login response, no separate Cloudant auth step.

The app syncs via **PouchDB replication** (`_revs_diff` → `_bulk_docs` → `_changes` longpoll).

**Known document IDs:** `profile`, `settings`, `stats`, `achievements`, `purchases`,
`gyms`, `analytics`, `exercise-notes`, `timer-durations`, `sharing-codes`,
`measurement-bodyweight`, `measurement-bodyfat`, `measurement-neck`,
`measurement-shoulders`, `measurement-chest`, `measurement-waist`,
`measurement-upper-arms-left/right`, `measurement-forearms-left/right`,
`measurement-calves-left/right`, plus workout docs with IDs like `{timestamp_ms}-{random}`.

**Workout document shape:**
```json
{
  "_id": "1788882599948-iajevnjn",
  "name": "Standalone workout",
  "exercises": [{
    "_id": "...",
    "exerciseId": "exercise-50",
    "repsTarget": 10,
    "sets": [{"reps": 10, "weight": 12.5, "weightUnit": "kg", "completed": 1788882608476}]
  }]
}
```

### Firebase

Used only for Cloud Functions (custom token):
`POST https://europe-central2-alphaprogression.cloudfunctions.net/getFirebaseCustomToken`
Body: `{"data":{"uuid":"<uuid>"}}`
Firebase API key: `AIzaSyAy0qvMPnctuyJl9IqhTd5T4xHZgAMIn98`, project: `alphaprogression`

---

## Key implementation details

- `src/crypto.rs` — password hashing and Cloudant Basic auth header construction
- `src/auth.rs` — `register_with_uuid` (WASM+native), `register` (native only, auto-generates UUID)
- `src/cloudant.rs` — `get_document<T>`, `bulk_get_raw`, `bulk_write_raw`, typed helpers
- UUID format: 64-char lowercase hex string (32 random bytes), NOT a standard UUID v4

**reqwest version note:** In reqwest 0.13 the TLS feature is `rustls` (not `rustls-tls`),
and `.form()` requires the `form` feature explicitly.

**wasm-bindgen init API:** Pass `{ module_or_path: buffer }` (object), not a bare buffer,
to avoid the "deprecated parameters" console warning.

---

## Building

```bash
# Native
cargo build -p alpha-progression
cargo test -p alpha-progression -- --ignored   # integration tests (hits real API)

# WASM library (for use from other Rust/WASM projects)
cargo build -p alpha-progression --target wasm32-unknown-unknown

# JS/WASM package — MUST use --target no-modules for file:// compatibility
wasm-pack build alpha-progression-wasm --target no-modules

# Self-contained HTML (no server needed, works on file://)
python3 bundle.py
# => example/index.bundle.html  (gitignored — embeds the keys via the compiled WASM)
```

`--target web` cannot be used for the standalone HTML because ES module `import`
is blocked on `file://`. The `no-modules` target emits a plain IIFE (`wasm_bindgen` global).
`bundle.py` base64-encodes the WASM binary and inlines it so there are zero external fetches.

---

## Secrets and the test account

Nothing extracted from the app is committed. `.env` (gitignored) holds the five
`apiKey` values, the password salt, and the test account credentials
(`AP_TEST_EMAIL`, `AP_TEST_PASSWORD`, `AP_TEST_UUID`); `.env.example` documents the
shape. `alpha-progression/build.rs` forwards them to `env!()` at compile time, so the
same mechanism works for wasm32 builds. Real env vars override `.env`.

Read credentials from `.env` — never paste them into source, docs, or commit messages.

The full API reference (all routes, headers, request/response bodies) is in `docs/alpha-progression-reference.md`.
Not public information may be available in `docs/captures`.