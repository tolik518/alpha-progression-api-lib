# Alpha Progression API Reference

> The `apiKey` values and the password salt below are shown as placeholders. They are
> not published here — extract them from the app yourself and put them in `.env`.
> See [extracting-api-keys.md](extracting-api-keys.md).

## Auth concepts

### Password hashing
All passwords are hashed client-side before sending. The salt is a static 64-char hex
string, identical for every user, inlined in the app bundle.
```
SHA256(password + $AP_PASSWORD_SALT)
```
```bash
echo -n "yourpassword$AP_PASSWORD_SALT" | sha256sum
```

### UUID
Client-generated 64-char hex string. Created once at registration, stored locally.

### Session token
Returned by register/login. Used as the Cloudant password.

### Cloudant auth
```
Authorization: Basic base64({uuid}:{session_token})
```

---

## 1. Register

```
POST https://alphaprogression.com/api/v3/user-signup
Content-Type: application/x-www-form-urlencoded
```

**Body**
```
apiKey=$AP_API_KEY_SIGNUP
&name={uuid}
&email={email}
&password={sha256(password+salt)}
&lang=en
&gender=male
&currency=
&country=
```

**Response 200**
```json
{"success":true,"token":"c292573ef3d0f9731f7feb169f0ca6cbc542df11424e0d0977e293c52690689f"}
```
> Note: `name` (uuid) is not returned here — you need to supply it yourself and store it.

---

## 2. Login

```
POST https://alphaprogression.com/api/v3/user-login
Content-Type: application/x-www-form-urlencoded
```

**Body**
```
apiKey=$AP_API_KEY_LOGIN
&email={email}
&password={sha256(password+salt)}
```

**Response 200**
```json
{"success":true,"name":"{uuid}","token":"{session_token}"}
```

---

## 3. Track month (telemetry, call after login)

```
POST https://alphaprogression.com/api/v3/track-month
Content-Type: application/x-www-form-urlencoded
```

**Body**
```
apiKey=$AP_API_KEY_TRACK_MONTH
&name={uuid}
&email={email}
&timezoneOffset=-120
```

**Response 200**
```json
{"success":"user_saved"}
```

---

## 4. Check subscription

```
POST https://alphaprogression.com/api/v3/check-subscription?v=7.5&u={first_16_chars_of_uuid}
Content-Type: application/x-www-form-urlencoded
```

**Body**
```
apiKey=$AP_API_KEY_CHECK_SUBSCRIPTION
&email={email}
```

**Response 200 (no subscription)**
```
false
```

**Response 200 (subscribed)**
```json
{"active":true, ...}
```

---

## 5. Set currency

```
POST https://alphaprogression.com/api/v3/set-currency
Content-Type: application/x-www-form-urlencoded
```

**Body**
```
apiKey=$AP_API_KEY_SET_CURRENCY
&name={uuid}
&email={email}
&currency=EUR
&country=DE
```

---

## 6. Get Firebase custom token

```
POST https://europe-central2-alphaprogression.cloudfunctions.net/getFirebaseCustomToken
Content-Type: application/json
```

**Body**
```json
{"data":{"uuid":"{uuid}"}}
```

**Response 200**
```json
{"result":"{firebase_custom_token}"}
```

---

## 7. Cloudant (user data database)

Base URL: `https://8e4fc8f1-3681-4dc2-8215-45ad5f9233e6-bluemix.cloudant.com`  
Database name = `{uuid}` (one DB per user)

**All requests require:**
```
Authorization: Basic {base64({uuid}:{session_token})}
Content-Type: application/json
Accept: application/json
```

### 7.1 Check server

```
GET /
```

**Response 200**
```json
{"couchdb":"Welcome","version":"3.5.2+cloudant","vendor":{"name":"IBM Cloudant","version":"8719","variant":"paas"}}
```

### 7.2 Get database info

```
GET /{uuid}/
```

**Response 200**
```json
{
  "db_name": "{uuid}",
  "doc_count": 27,
  "doc_del_count": 0,
  "update_seq": "...",
  "sizes": {"file": 12470, "external": 0, "active": 0}
}
```

### 7.3 Get changes feed (initial sync)

```
GET /{uuid}/_changes?style=all_docs&since=0&limit=100
```

**Response 200**
```json
{
  "results": [
    {"id": "profile", "seq": "...", "changes": [{"rev": "1-abc..."}]},
    {"id": "settings", "seq": "...", "changes": [{"rev": "2-def..."}]},
    ...
  ],
  "last_seq": "...",
  "pending": 0
}
```

### 7.4 Get changes feed (long-poll, live updates)

```
GET /{uuid}/_changes?style=all_docs&feed=longpoll&heartbeat=10000&since={last_seq}&limit=100
```

### 7.5 Check revision diffs (PouchDB sync step 1)

```
POST /{uuid}/_revs_diff
```

**Body** — map of doc ID → array of known revisions
```json
{
  "profile": ["1-b28bc4b0eda84ce1967d4b9e28d7548f"],
  "settings": ["2-b29a81a642dda153c12cef43b85ef3b7"],
  "achievements": ["6-fd0ae55c214e51d7aaa9b76f4ed8387e"],
  "measurement-bodyweight": ["1-edd8c16a5ee38c9e6baffc5b325359e7"],
  "measurement-bodyfat": ["1-7d2fe0d1feca91f0fe78b1e08e1f728c"],
  ...
}
```

**Response 200** — only docs where server revision differs
```json
{
  "achievements": {"missing": ["6-fd0ae55c214e51d7aaa9b76f4ed8387e"]},
  "settings": {"missing": ["2-b29a81a642dda153c12cef43b85ef3b7"]},
  "stats": {"missing": ["1-7850ade769d591f6c4b3fd375311700c"]}
}
```

### 7.6 Bulk write documents (PouchDB sync step 2)

```
POST /{uuid}/_bulk_docs
```

**Body**
```json
{
  "docs": [
    {
      "_id": "profile",
      "_rev": "1-abc...",
      "name": "...",
      "email": "..."
    },
    {
      "_id": "settings",
      "_rev": "2-def...",
      "weightUnit": "kg",
      ...
    },
    {
      "_id": "1788882599948-iajevnjn",
      "name": "Standalone workout",
      "exercises": [
        {
          "_id": "1788882599945",
          "exerciseId": "exercise-50",
          "repsTarget": 10,
          "sets": [
            {"reps": 10, "weight": 12.5, "weightUnit": "kg", "completed": 1788882608476}
          ]
        }
      ]
    }
  ]
}
```

**Response 201**
```json
[]
```
> Empty array = all succeeded. Errors appear as objects in the array.

### 7.7 Bulk read documents

```
POST /{uuid}/_bulk_get
```

**Body**
```json
{
  "docs": [
    {"id": "profile"},
    {"id": "settings"},
    {"id": "achievements"},
    {"id": "1788882599948-iajevnjn"}
  ]
}
```

**Response 200**
```json
{
  "results": [
    {
      "id": "profile",
      "docs": [{"ok": {"_id": "profile", "_rev": "1-...", ...}}]
    }
  ]
}
```

### 7.8 PouchDB replication checkpoint

```
GET  /{uuid}/_local/{checkpoint_id}
PUT  /{uuid}/_local/{checkpoint_id}
```
Internal PouchDB bookkeeping, not needed for manual API access.

---

## Known document IDs in user database

| `_id` | Content |
|-------|---------|
| `profile` | User profile (name, email, gender) |
| `settings` | App settings (weight unit, theme, etc.) |
| `stats` | Usage stats |
| `achievements` | Progress on all achievement challenges |
| `purchases` | Subscription/purchase state |
| `gyms` | Gym list |
| `analytics` | Analytics data |
| `exercise-notes` | Per-exercise user notes |
| `timer-durations` | Custom rest timer settings |
| `sharing-codes` | Workout sharing codes |
| `measurement-bodyweight` | Body weight history |
| `measurement-bodyfat` | Body fat % history |
| `measurement-neck` | Neck circumference history |
| `measurement-shoulders` | Shoulders circumference history |
| `measurement-chest` | Chest circumference history |
| `measurement-waist` | Waist circumference history |
| `measurement-upper-arms-left` | Left upper arm history |
| `measurement-upper-arms-right` | Right upper arm history |
| `measurement-forearms-left` | Left forearm history |
| `measurement-forearms-right` | Right forearm history |
| `measurement-calves-left` | Left calf history |
| `measurement-calves-right` | Right calf history |
| `{timestamp}-{random}` | Workout document (e.g. `1788882599948-iajevnjn`) |

---

## Firebase config (for reference)

```
apiKey:    AIzaSyAy0qvMPnctuyJl9IqhTd5T4xHZgAMIn98
projectId: alphaprogression
```
