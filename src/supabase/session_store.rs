use std::collections::HashMap;

use axum::async_trait;
use axum_login::tower_sessions::{session, session_store, SessionStore};
use leptos::serde_json;
use supabase_rust::errors::{AuthError, Error, ErrorKind, JwtErrorKind, PostgrestError};
use supabase_rust::schema::AccessToken;
use time::format_description::well_known::Iso8601;
use time::OffsetDateTime;

use super::{AppUser, IdentityData, SupabaseBackend};

#[derive(serde::Serialize, serde::Deserialize, Debug, Default)]
struct SessionSchema {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sb_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refreshed_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expiry_date: Option<String>,
}

impl SupabaseBackend {
    pub async fn new_session(&self, access_token: AccessToken) -> Result<AppUser, Error> {
        let claims = self
            .client
            .jwt_valid(&access_token.access_token)
            .await
            .map_err(|e| Error {
                http_status: 403,
                kind: ErrorKind::Auth(AuthError {
                    msg: Some(format!("Invalid Token {e:?}")),
                    ..Default::default()
                }),
            })?;

        let query = self
            .client
            .query()
            .from("sessions")
            .eq("sb_id", claims.session_id.clone())
            .select("expiry_date")
            .auth(&self.service_key)
            .execute()
            .await;

        match supabase_rust::parse_response::<SessionSchema>(query).await {
            Ok(sessions) if !sessions.is_empty() => {
                let expiry_date = IdentityData::parse_expiry_date(&sessions[0].expiry_date);

                let identity = IdentityData {
                    auth_token: access_token.access_token,
                    refresh_token: access_token.refresh_token.clone(),
                    expiry_date,
                    last_accessed: OffsetDateTime::now_utc(),
                    user_id: access_token.user.id.clone(),
                    email: access_token.user.email.clone(),
                    has_mfa: false,
                    aal: claims.aal,
                };

                let user = AppUser { identity };

                Ok(user)
            }
            Ok(_) => Err(Error {
                http_status: 404,
                kind: ErrorKind::Postgrest(PostgrestError {
                    message: "Found {{0}} rows matching the query".to_string(),
                    ..Default::default()
                }),
            }),
            Err(e) => Err(e),
        }
    }

    async fn validate_session(
        &self,
        mut session_record: session::Record,
    ) -> Option<(session::Record, bool)> {
        let mut dirty = false;
        let mut identity = IdentityData::from_record_data(&session_record.data)?;
        let token_validation = self.client.jwt_valid(&identity.auth_token).await;

        match token_validation {
            Ok(_) => {
                if identity.should_refetch_user() {
                    if self.client.get_user(&identity.auth_token).await.is_err() {
                        return self.refresh_session_token(identity, session_record).await;
                    }
                    identity.last_accessed = OffsetDateTime::now_utc();
                    session_record.data = identity.into_record_data();
                    dirty = true;
                }
                Some((session_record, dirty))
            }
            Err(e) => {
                if e.into_kind() == JwtErrorKind::ExpiredSignature {
                    return self.refresh_session_token(identity, session_record).await;
                }
                None
            }
        }
    }

    async fn refresh_session_token(
        &self,
        mut identity: IdentityData,
        mut session_record: session::Record,
    ) -> Option<(session::Record, bool)> {
        if let Ok(access) = self.client.refresh_token(&identity.refresh_token).await {
            identity.auth_token = access.access_token;
            identity.refresh_token = access.refresh_token;
            identity.last_accessed = OffsetDateTime::now_utc();
            session_record.data = identity.into_record_data();
            return Some((session_record, true));
        }
        None
    }

    pub async fn update_assurance_level(
        &self,
        session_id: &str,
        mut identity: IdentityData,
        token: AccessToken,
    ) {
        let claims = self
            .client
            .jwt_valid(&token.access_token)
            .await
            .ok()
            .unwrap();
        identity.last_accessed = OffsetDateTime::now_utc();
        identity.refresh_token = token.refresh_token.clone();
        identity.auth_token = token.access_token;
        identity.aal = claims.aal;
        let _ = self.save(&identity.into_session_record(session_id)).await;
    }
}

#[async_trait]
impl SessionStore for SupabaseBackend {
    async fn save(&self, record: &session::Record) -> session_store::Result<()> {
        self.sessions_cache.save(record).await?;

        let session_schema = serde_json::to_string(&SessionSchema {
            id: Some(record.id.to_string()),
            data: Some(serde_json::to_string(&record.data).unwrap_or_default()),
            refreshed_at: Some(OffsetDateTime::now_utc().format(&Iso8601::DEFAULT).unwrap()),
            expiry_date: Some(record.expiry_date.format(&Iso8601::DEFAULT).unwrap()),
            ..Default::default()
        })
        .unwrap();

        let query = self
            .client
            .query()
            .from("sessions")
            .update(session_schema)
            .eq("id", record.id.to_string())
            .auth(&self.service_key)
            .execute()
            .await;

        if query.is_err() {
            tracing::error!("\nSessionStore.save is_err");
        }
        Ok(())
    }

    async fn load(&self, id: &session::Id) -> session_store::Result<Option<session::Record>> {
        let cached = match self.sessions_cache.load(id).await {
            Ok(Some(session_record)) => {
                if let Some((record, dirty)) = self.validate_session(session_record).await {
                    if dirty {
                        self.save(&record).await?;
                    }
                    Some(record)
                } else {
                    None
                }
            }
            Ok(None) => None,
            Err(e) => {
                tracing::error!("\nSessionStore.load cache- {e:?}");
                None
            }
        };

        if let Some(record) = cached {
            return Ok(Some(record));
        }

        let query = self
            .client
            .query()
            .from("sessions")
            .select("data,expiry_date")
            .eq("id", id.to_string())
            .single()
            .auth(&self.service_key)
            .execute()
            .await;

        let sessions = supabase_rust::parse_response::<SessionSchema>(query)
            .await
            .map_err(|e| {
                tracing::error!("\nSessionStore.load - {e:?}");
            })
            .ok()
            .unwrap_or_default();

        if sessions.is_empty() {
            let _ = self.delete(id).await;
            return Ok(None);
        }

        let expiry_date = IdentityData::parse_expiry_date(&sessions[0].expiry_date);
        let data = sessions[0]
            .data
            .as_ref()
            .and_then(|data_str| {
                serde_json::from_str::<HashMap<String, serde_json::Value>>(data_str)
                    .map_err(|e| {
                        tracing::error!("\nSessionStore.load.parse_data - {e:?}");
                        e
                    })
                    .ok()
            })
            .unwrap_or_default();

        let session_record = session::Record {
            id: *id,
            data,
            expiry_date,
        };

        match self.validate_session(session_record).await {
            Some((record, dirty)) => {
                if dirty {
                    let _ = self.save(&record).await;
                } else {
                    self.sessions_cache.save(&record).await?;
                }
                Ok(Some(record))
            }
            None => {
                let _ = self.delete(id).await;
                Ok(None)
            }
        }
    }

    async fn delete(&self, id: &session::Id) -> session_store::Result<()> {
        self.sessions_cache.delete(id).await?;

        let _ = self
            .client
            .query()
            .from("sessions")
            .delete()
            .eq("id", id.to_string())
            .auth(&self.service_key)
            .execute()
            .await;
        Ok(())
    }

    async fn create(&self, record: &mut session::Record) -> session_store::Result<()> {
        let Some(identity) = IdentityData::from_record_data(&record.data) else {
            return Ok(());
        };
        let claims = self.client.jwt_valid(&identity.auth_token).await.unwrap();

        self.sessions_cache.create(record).await?;

        let session_schema = serde_json::to_string(&SessionSchema {
            id: Some(record.id.to_string()),
            data: Some(serde_json::to_string(&record.data).unwrap_or_default()),
            expiry_date: Some(record.expiry_date.format(&Iso8601::DEFAULT).unwrap()),
            ..Default::default()
        })
        .unwrap();

        let query = self
            .client
            .query()
            .from("sessions")
            // When authenticated a trigger inserts a new entry into the table so we UPDATE here
            .update(session_schema)
            .eq("sb_id", claims.session_id)
            .auth(&self.service_key)
            .execute()
            .await;

        if query.is_err() {
            tracing::error!("\nSessionStore.create is_err");
        }
        Ok(())
    }
}
