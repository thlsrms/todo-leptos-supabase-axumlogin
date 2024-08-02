use std::collections::HashMap;
use std::str::FromStr;

use axum_login::tower_sessions::session;
use leptos::serde_json;
use time::ext::NumericalDuration;
use time::format_description::well_known::Iso8601;
use time::OffsetDateTime;

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
pub struct IdentityData {
    pub auth_token: String,
    pub refresh_token: String,
    pub expiry_date: OffsetDateTime,
    pub last_accessed: OffsetDateTime,
    pub user_id: String,
    pub email: String,
    pub has_mfa: bool,
    pub aal: String,
}

impl IdentityData {
    pub fn from_record_data(data: &HashMap<String, serde_json::Value>) -> Option<Self> {
        match serde_json::from_value::<IdentityData>(
            data.get("axum-login.data")
                .unwrap()
                .get("user_id")
                .unwrap()
                .clone(),
        ) {
            Ok(identity) => Some(identity),
            Err(e) => {
                tracing::error!("IdentityData parse err: {e:?}");
                None
            }
        }
    }

    pub fn into_record_data(self) -> HashMap<String, serde_json::Value> {
        use serde_json::{json, Map, Value};
        let mut im = Map::new();
        im.insert(
            "auth_hash".to_string(),
            self.refresh_token.as_bytes().into(),
        );
        im.insert("user_id".to_string(), json!(self));
        let mut hm = HashMap::new();
        hm.insert("axum-login.data".to_string(), Value::Object(im));
        hm
    }

    pub fn into_session_record(self, id: &str) -> session::Record {
        session::Record {
            id: session::Id::from_str(id).unwrap(),
            expiry_date: self.expiry_date,
            data: self.into_record_data(),
        }
    }

    pub fn parse_expiry_date(expiry_date: &Option<String>) -> OffsetDateTime {
        expiry_date
            .as_ref()
            .and_then(|date_str| OffsetDateTime::parse(date_str, &Iso8601::DEFAULT).ok())
            .unwrap_or_else(|| OffsetDateTime::now_utc() + time::Duration::days(1))
    }

    pub fn should_refetch_user(&self) -> bool {
        OffsetDateTime::now_utc() > self.last_accessed.saturating_add(5.minutes())
    }
}

impl std::fmt::Display for IdentityData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "IdentityData {{ auth_token: '{}', refresh_token: '{}', \
                expiry_date: '{}', last_accessed: '{}', user_id: '{}', email: '{}' }}",
            self.auth_token,
            self.refresh_token,
            self.expiry_date,
            self.last_accessed,
            self.user_id,
            self.email
        )
    }
}
