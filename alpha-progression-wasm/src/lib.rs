use alpha_progression::{AlphaProgressionClient, MeasurementKind, Session};
use wasm_bindgen::prelude::*;

// Converts any Display error into a JsValue for JS rejection.
fn to_js_err(e: impl std::fmt::Display) -> JsValue {
    JsValue::from_str(&e.to_string())
}

fn session_from_js(val: JsValue) -> Result<Session, JsValue> {
    serde_wasm_bindgen::from_value(val).map_err(|e| to_js_err(e))
}

fn to_js<T: serde::Serialize>(val: &T) -> Result<JsValue, JsValue> {
    serde_wasm_bindgen::to_value(val).map_err(|e| to_js_err(e))
}

/// Generate a random 64-char hex user ID using the browser's crypto API.
#[wasm_bindgen]
pub fn generate_user_id() -> String {
    use rand::RngCore;
    let mut bytes = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut bytes);
    hex::encode(bytes)
}

#[wasm_bindgen]
pub struct Client {
    inner: AlphaProgressionClient,
}

#[wasm_bindgen]
impl Client {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            inner: AlphaProgressionClient::new(),
        }
    }

    // -------------------------------------------------------------------------
    // Auth
    // -------------------------------------------------------------------------

    /// Register a new account. Generates a UUID automatically.
    pub async fn register(&self, email: &str, password: &str) -> Result<JsValue, JsValue> {
        let uuid = generate_user_id();
        let session = self
            .inner
            .register_with_uuid(&uuid, email, password)
            .await
            .map_err(|e| to_js_err(e))?;
        to_js(&session)
    }

    /// Register with an explicit UUID (for restoring an account).
    pub async fn register_with_uuid(
        &self,
        uuid: &str,
        email: &str,
        password: &str,
    ) -> Result<JsValue, JsValue> {
        let session = self
            .inner
            .register_with_uuid(uuid, email, password)
            .await
            .map_err(|e| to_js_err(e))?;
        to_js(&session)
    }

    pub async fn login(&self, email: &str, password: &str) -> Result<JsValue, JsValue> {
        let session = self
            .inner
            .login(email, password)
            .await
            .map_err(|e| to_js_err(e))?;
        to_js(&session)
    }

    // -------------------------------------------------------------------------
    // REST API
    // -------------------------------------------------------------------------

    pub async fn track_month(
        &self,
        session: JsValue,
        timezone_offset: i32,
    ) -> Result<(), JsValue> {
        let s = session_from_js(session)?;
        self.inner
            .track_month(&s, timezone_offset)
            .await
            .map_err(|e| to_js_err(e))
    }

    /// Returns `true` if subscribed, `false` otherwise.
    pub async fn check_subscription(&self, session: JsValue) -> Result<bool, JsValue> {
        let s = session_from_js(session)?;
        self.inner
            .check_subscription(&s)
            .await
            .map_err(|e| to_js_err(e))
    }

    pub async fn set_currency(
        &self,
        session: JsValue,
        currency: &str,
        country: &str,
    ) -> Result<(), JsValue> {
        let s = session_from_js(session)?;
        self.inner
            .set_currency(&s, currency, country)
            .await
            .map_err(|e| to_js_err(e))
    }

    // -------------------------------------------------------------------------
    // Cloudant
    // -------------------------------------------------------------------------

    pub async fn db_info(&self, session: JsValue) -> Result<JsValue, JsValue> {
        let s = session_from_js(session)?;
        let info = self.inner.db_info(&s).await.map_err(|e| to_js_err(e))?;
        to_js(&info)
    }

    /// Returns `{ results, last_seq, pending }`.
    pub async fn get_changes(
        &self,
        session: JsValue,
        since: &str,
        limit: u32,
    ) -> Result<JsValue, JsValue> {
        let s = session_from_js(session)?;
        let changes = self
            .inner
            .get_changes(&s, since, limit)
            .await
            .map_err(|e| to_js_err(e))?;
        to_js(&changes)
    }

    pub async fn get_workout(
        &self,
        session: JsValue,
        doc_id: &str,
    ) -> Result<JsValue, JsValue> {
        let s = session_from_js(session)?;
        let doc = self
            .inner
            .get_workout(&s, doc_id)
            .await
            .map_err(|e| to_js_err(e))?;
        to_js(&doc)
    }

    pub async fn put_workout(&self, session: JsValue, workout: JsValue) -> Result<(), JsValue> {
        let s = session_from_js(session)?;
        let doc: alpha_progression::WorkoutDoc =
            serde_wasm_bindgen::from_value(workout).map_err(|e| to_js_err(e))?;
        self.inner
            .put_workout(&s, &doc)
            .await
            .map_err(|e| to_js_err(e))
    }

    /// `kind`: one of "bodyweight", "bodyfat", "neck", "shoulders", "chest",
    /// "waist", "upper-arms-left", "upper-arms-right", "forearms-left",
    /// "forearms-right", "calves-left", "calves-right"
    pub async fn get_measurements(
        &self,
        session: JsValue,
        kind: &str,
    ) -> Result<JsValue, JsValue> {
        let s = session_from_js(session)?;
        let measurement_kind = parse_measurement_kind(kind)?;
        let doc = self
            .inner
            .get_measurements(&s, measurement_kind)
            .await
            .map_err(|e| to_js_err(e))?;
        to_js(&doc)
    }

    pub async fn bulk_get_raw(
        &self,
        session: JsValue,
        doc_ids: Vec<String>,
    ) -> Result<JsValue, JsValue> {
        let s = session_from_js(session)?;
        let ids: Vec<&str> = doc_ids.iter().map(String::as_str).collect();
        let docs = self
            .inner
            .bulk_get_raw(&s, &ids)
            .await
            .map_err(|e| to_js_err(e))?;
        to_js(&docs)
    }
}

fn parse_measurement_kind(kind: &str) -> Result<MeasurementKind, JsValue> {
    match kind {
        "bodyweight"      => Ok(MeasurementKind::Bodyweight),
        "bodyfat"         => Ok(MeasurementKind::Bodyfat),
        "neck"            => Ok(MeasurementKind::Neck),
        "shoulders"       => Ok(MeasurementKind::Shoulders),
        "chest"           => Ok(MeasurementKind::Chest),
        "waist"           => Ok(MeasurementKind::Waist),
        "upper-arms-left"  => Ok(MeasurementKind::UpperArmsLeft),
        "upper-arms-right" => Ok(MeasurementKind::UpperArmsRight),
        "forearms-left"   => Ok(MeasurementKind::ForeArmsLeft),
        "forearms-right"  => Ok(MeasurementKind::ForeArmsRight),
        "calves-left"     => Ok(MeasurementKind::CalvesLeft),
        "calves-right"    => Ok(MeasurementKind::CalvesRight),
        other => Err(to_js_err(format!("unknown measurement kind: {}", other))),
    }
}
