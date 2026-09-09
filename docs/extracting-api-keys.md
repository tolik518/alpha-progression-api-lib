# Extracting the API keys

The app authenticates each endpoint with a hardcoded `apiKey`, and salts passwords with
a static app-wide value before hashing. Those values are not included in this
repository — you have to pull them out of the app yourself, then put them in `.env`.

This crate needs five keys, plus the password salt:

| Variable | Used by |
| --- | --- |
| `AP_API_KEY_SIGNUP` | `POST /api/v3/user-signup` |
| `AP_API_KEY_LOGIN` | `POST /api/v3/user-login` |
| `AP_API_KEY_TRACK_MONTH` | `POST /api/v3/track-month` |
| `AP_API_KEY_CHECK_SUBSCRIPTION` | `POST /api/v3/check-subscription` |
| `AP_API_KEY_SET_CURRENCY` | `POST /api/v3/set-currency` |
| `AP_PASSWORD_SALT` | appended to the password before `SHA256` |

Each key is a 64-character lowercase hex string. They are the same for every user —
they identify the app, not you. The app ships about fifteen in total; the extraction
script below prints all of them, mapped to their endpoints.

Two ways to get them. The script is the quick one — traffic capture is worth doing
anyway if you want to see the requests themselves.

## Option A: capture your own traffic

1. Install the Android app on a device or emulator you control.
2. Point it at an HTTPS-intercepting proxy — [HTTP Toolkit](https://httptoolkit.com)
   or mitmproxy — with the proxy's CA certificate trusted by the device.
3. Exercise the app: register an account, log in, change the currency in settings.
4. Each request to `https://alphaprogression.com/api/v3/<endpoint>` is a
   `application/x-www-form-urlencoded` POST whose body starts with `apiKey=<64 hex>`.
   Read the key straight out of the body and match it to the endpoint in the URL.

Some keys only appear when the matching action fires: `user-signup` on registration,
`set-currency` when you change currency, `track-month` on first launch in a month.

The salt is not visible this way — the app hashes before sending — so pull it from the
bundle as described below.

## Option B: pull them out of the APK (recommended)

1. Get the APK off a device:

   ```bash
   adb shell pm path com.alphaprogression.alphaprogression
   adb pull /data/app/.../base.apk
   ```

2. It's a Capacitor (Ionic) app, so the web assets are plain files inside the zip:

   ```bash
   unzip -o base.apk -d base_extracted    # or: apktool d base.apk
   ```

3. All the business logic is in a single bundle, `assets/public/assets/index-<hash>.js`
   (~3 MB). The hash changes with every release.

4. Run [../tools/extract-keys.mjs](../tools/extract-keys.mjs) against it — node or
   deno, no dependencies. It prints every key with the endpoint it belongs to:

   ```bash
   node tools/extract-keys.mjs base_extracted/assets/public/assets/index-*.js
   ```

   ```text
   /api/v3/check-subscription                   e4122a6b55…
   /api/v3/fix-permissions, /api/v3/user-signup 89f5452b79…
   /api/v3/set-currency                         691040b44e…
   /api/v3/track-month                          e3c3ca462d…
   /api/v3/user-login                           0bd62c6369…
   …
   password salt                                460bd0d792…
   ```

   Copy the values into `.env`. Note that one key covers both `/user-signup` and
   `/fix-permissions` — that's the app's doing, not a bug in the script.

### Why it needs a script and not a grep

A plain `grep -o "'[0-9a-f]\{64\}'" index-*.js | sort -u` does give you 17 candidate
strings, and the five keys are among them — but nothing says which is which, so it
isn't much use on its own.

The bundle is obfuscated with string-array indirection. Call sites read
`{'apiKey':_0x2a8f49(0x41e),'email':…}`: the literal isn't next to the URL, it's at
index `0x41e` of a shared string table. Worse, a preamble at the top of the file
rotates that table into position at startup, so even indexing it statically gives you
the wrong entry.

Rather than reimplement any of that, the script lifts three pieces out of the bundle —
the string array, the rotation preamble, and the decoder function — re-runs just those
in isolation, and then calls the app's own decoder to resolve each index. That also
means it keeps working across releases, where hardcoded offsets wouldn't.

The salt is the exception: it's inline at the `sha256` call, so it greps out directly.

```bash
grep -o "sha256(.\{0,140\}" index-*.js | grep -o "'[0-9a-f]\{64\}'"
```
