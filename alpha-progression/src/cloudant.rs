use serde::de::DeserializeOwned;
use serde_json::json;

use crate::{
    client::{AlphaProgressionClient, CLOUDANT_BASE},
    crypto,
    error::{Error, Result},
    models::{
        ChangesResponse, DbInfo, MeasurementDoc, MeasurementKind, Session, WorkoutDoc,
    },
};

impl AlphaProgressionClient {
    fn cloudant_url(&self, session: &Session, path: &str) -> String {
        format!("{}/{}/{}", CLOUDANT_BASE, session.uuid, path)
    }

    fn auth_header(&self, session: &Session) -> String {
        crypto::basic_auth(&session.uuid, &session.token)
    }

    pub async fn db_info(&self, session: &Session) -> Result<DbInfo> {
        let resp = self
            .http
            .get(format!("{}/{}/", CLOUDANT_BASE, session.uuid))
            .header("Authorization", self.auth_header(session))
            .send()
            .await?;
        Ok(resp.json().await?)
    }

    pub async fn get_changes(
        &self,
        session: &Session,
        since: &str,
        limit: u32,
    ) -> Result<ChangesResponse> {
        let url = self.cloudant_url(
            session,
            &format!("_changes?style=all_docs&since={}&limit={}", since, limit),
        );
        let resp = self
            .http
            .get(&url)
            .header("Authorization", self.auth_header(session))
            .header("Content-Type", "application/json")
            .send()
            .await?;
        Ok(resp.json().await?)
    }

    pub async fn get_document<T: DeserializeOwned>(
        &self,
        session: &Session,
        doc_id: &str,
    ) -> Result<T> {
        let url = self.cloudant_url(session, doc_id);
        let resp = self
            .http
            .get(&url)
            .header("Authorization", self.auth_header(session))
            .header("Accept", "application/json")
            .send()
            .await?;

        if resp.status() == 404 {
            return Err(Error::NotFound(doc_id.to_string()));
        }

        let body: serde_json::Value = resp.json().await?;
        if let Some(err) = body.get("error") {
            return Err(Error::Api(err.to_string()));
        }

        Ok(serde_json::from_value(body)?)
    }

    pub async fn put_document(&self, session: &Session, doc: &serde_json::Value) -> Result<()> {
        let doc_id = doc["_id"]
            .as_str()
            .ok_or_else(|| Error::Api("document missing _id".into()))?;
        let url = self.cloudant_url(session, doc_id);
        let resp = self
            .http
            .put(&url)
            .header("Authorization", self.auth_header(session))
            .header("Content-Type", "application/json")
            .json(doc)
            .send()
            .await?;

        let body: serde_json::Value = resp.json().await?;
        if body["ok"].as_bool() != Some(true) {
            return Err(Error::Api(body.to_string()));
        }
        Ok(())
    }

    pub async fn bulk_get_raw(
        &self,
        session: &Session,
        doc_ids: &[&str],
    ) -> Result<Vec<serde_json::Value>> {
        let url = self.cloudant_url(session, "_bulk_get");
        let body = json!({
            "docs": doc_ids.iter().map(|id| json!({"id": id})).collect::<Vec<_>>()
        });

        let resp = self
            .http
            .post(&url)
            .header("Authorization", self.auth_header(session))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await?;

        let result: serde_json::Value = resp.json().await?;
        let docs = result["results"]
            .as_array()
            .ok_or_else(|| Error::Api("unexpected _bulk_get response".into()))?
            .iter()
            .filter_map(|entry| {
                entry["docs"]
                    .as_array()?
                    .first()?
                    .get("ok")
                    .cloned()
            })
            .collect();

        Ok(docs)
    }

    pub async fn bulk_write_raw(
        &self,
        session: &Session,
        docs: Vec<serde_json::Value>,
    ) -> Result<()> {
        let url = self.cloudant_url(session, "_bulk_docs");
        let body = json!({ "docs": docs });

        let resp = self
            .http
            .post(&url)
            .header("Authorization", self.auth_header(session))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await?;

        let result: serde_json::Value = resp.json().await?;
        if let Some(errors) = result.as_array() {
            let failed: Vec<_> = errors
                .iter()
                .filter(|e| e.get("error").is_some())
                .collect();
            if !failed.is_empty() {
                return Err(Error::Api(format!(
                    "{} document(s) failed to write",
                    failed.len()
                )));
            }
        }
        Ok(())
    }

    // ---------------------------------------------------------------------------
    // Typed helpers
    // ---------------------------------------------------------------------------

    pub async fn get_workout(&self, session: &Session, doc_id: &str) -> Result<WorkoutDoc> {
        self.get_document(session, doc_id).await
    }

    pub async fn put_workout(&self, session: &Session, workout: &WorkoutDoc) -> Result<()> {
        let value = serde_json::to_value(workout)?;
        self.put_document(session, &value).await
    }

    pub async fn get_measurements(
        &self,
        session: &Session,
        kind: MeasurementKind,
    ) -> Result<MeasurementDoc> {
        self.get_document(session, kind.doc_id()).await
    }
}
