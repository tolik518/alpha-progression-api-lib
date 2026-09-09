use crate::{
    client::{AlphaProgressionClient, API_BASE},
    crypto,
    error::{Error, Result},
    models::Session,
};

const KEY_SIGNUP: &str = app_secret!("AP_API_KEY_SIGNUP");
const KEY_LOGIN: &str = app_secret!("AP_API_KEY_LOGIN");

impl AlphaProgressionClient {
    /// Generates a random UUID and registers. Native only — on WASM use `register_with_uuid`.
    #[cfg(not(target_arch = "wasm32"))]
    pub async fn register(&self, email: &str, password: &str) -> Result<Session> {
        let uuid = crypto::generate_user_id();
        self.register_with_uuid(&uuid, email, password).await
    }

    pub async fn register_with_uuid(
        &self,
        uuid: &str,
        email: &str,
        password: &str,
    ) -> Result<Session> {
        let hash = crypto::hash_password(password);
        let resp = self
            .http
            .post(format!("{}/user-signup", API_BASE))
            .form(&[
                ("apiKey", KEY_SIGNUP),
                ("name", uuid),
                ("email", email),
                ("password", &hash),
                ("lang", "en"),
                ("gender", "male"),
                ("currency", ""),
                ("country", ""),
            ])
            .send()
            .await?;

        let body: serde_json::Value = resp.json().await?;
        let token = body["token"]
            .as_str()
            .ok_or_else(|| Error::Api(body.to_string()))?
            .to_string();

        Ok(Session {
            uuid: uuid.to_string(),
            email: email.to_string(),
            token,
        })
    }

    pub async fn login(&self, email: &str, password: &str) -> Result<Session> {
        let hash = crypto::hash_password(password);
        let resp = self
            .http
            .post(format!("{}/user-login", API_BASE))
            .form(&[
                ("apiKey", KEY_LOGIN),
                ("email", email),
                ("password", &hash),
            ])
            .send()
            .await?;

        let body: serde_json::Value = resp.json().await?;
        if body["success"].as_bool() != Some(true) {
            return Err(Error::AuthFailed);
        }

        let uuid = body["name"]
            .as_str()
            .ok_or(Error::AuthFailed)?
            .to_string();
        let token = body["token"]
            .as_str()
            .ok_or(Error::AuthFailed)?
            .to_string();

        Ok(Session {
            uuid,
            email: email.to_string(),
            token,
        })
    }
}
