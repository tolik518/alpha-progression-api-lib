use crate::{
    client::{AlphaProgressionClient, API_BASE},
    error::Result,
    models::Session,
};

const KEY_TRACK_MONTH: &str = app_secret!("AP_API_KEY_TRACK_MONTH");
const KEY_CHECK_SUBSCRIPTION: &str = app_secret!("AP_API_KEY_CHECK_SUBSCRIPTION");
const KEY_SET_CURRENCY: &str = app_secret!("AP_API_KEY_SET_CURRENCY");

impl AlphaProgressionClient {
    pub async fn track_month(&self, session: &Session, timezone_offset: i32) -> Result<()> {
        let tz = timezone_offset.to_string();
        self.http
            .post(format!("{}/track-month", API_BASE))
            .form(&[
                ("apiKey", KEY_TRACK_MONTH),
                ("name", &session.uuid),
                ("email", &session.email),
                ("timezoneOffset", &tz),
            ])
            .send()
            .await?;
        Ok(())
    }

    pub async fn check_subscription(&self, session: &Session) -> Result<bool> {
        let uuid_prefix = &session.uuid[..16];
        let resp = self
            .http
            .post(format!(
                "{}/check-subscription?v=7.5&u={}",
                API_BASE, uuid_prefix
            ))
            .form(&[("apiKey", KEY_CHECK_SUBSCRIPTION), ("email", &session.email)])
            .send()
            .await?;

        let body: serde_json::Value = resp.json().await?;
        Ok(body.as_bool().unwrap_or(false)
            || body.get("active").and_then(|v| v.as_bool()).unwrap_or(false))
    }

    pub async fn set_currency(
        &self,
        session: &Session,
        currency: &str,
        country: &str,
    ) -> Result<()> {
        self.http
            .post(format!("{}/set-currency", API_BASE))
            .form(&[
                ("apiKey", KEY_SET_CURRENCY),
                ("name", &session.uuid),
                ("email", &session.email),
                ("currency", currency),
                ("country", country),
            ])
            .send()
            .await?;
        Ok(())
    }
}
