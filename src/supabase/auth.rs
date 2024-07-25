use axum::async_trait;
use axum_login::{AuthUser, AuthnBackend};

use super::{AppUser, IdentityData, SupabaseBackend};

impl AuthUser for AppUser {
    type Id = IdentityData;

    fn id(&self) -> Self::Id {
        self.identity.clone()
    }

    fn session_auth_hash(&self) -> &[u8] {
        self.identity.refresh_token.as_bytes()
    }
}

#[async_trait]
impl AuthnBackend for SupabaseBackend {
    type User = AppUser;
    type Credentials = (String, String);
    type Error = supabase_rust::errors::Error;

    async fn authenticate(
        &self,
        (email, password): Self::Credentials,
    ) -> Result<Option<Self::User>, Self::Error> {
        let access_token = self.client.sign_in_password(&email, &password).await?;
        Ok(Some(self.new_session(access_token).await?))
    }

    async fn get_user(
        &self,
        data: &axum_login::UserId<Self>,
    ) -> Result<Option<Self::User>, Self::Error> {
        Ok(Some(AppUser {
            identity: data.clone(),
        }))
    }
}
