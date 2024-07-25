use std::sync::Arc;

use axum::async_trait;
use axum_login::tower_sessions::{session, SessionStore};
use axum_login::AuthnBackend;

#[derive(Debug, Clone)]
pub struct StoreWrapper<T>(pub Arc<T>);

#[async_trait]
impl<T> SessionStore for StoreWrapper<T>
where
    T: SessionStore,
{
    async fn save(
        &self,
        session_record: &session::Record,
    ) -> axum_login::tower_sessions::session_store::Result<()> {
        self.0.save(session_record).await
    }

    async fn load(
        &self,
        session_id: &session::Id,
    ) -> axum_login::tower_sessions::session_store::Result<Option<session::Record>> {
        self.0.load(session_id).await
    }

    async fn delete(
        &self,
        session_id: &session::Id,
    ) -> axum_login::tower_sessions::session_store::Result<()> {
        self.0.delete(session_id).await
    }

    async fn create(
        &self,
        session_record: &mut session::Record,
    ) -> axum_login::tower_sessions::session_store::Result<()> {
        self.0.create(session_record).await
    }
}

#[derive(Debug, Clone)]
pub struct AuthWrapper<T>(pub Arc<T>);

#[async_trait]
impl<T> AuthnBackend for AuthWrapper<T>
where
    T: AuthnBackend,
{
    type User = T::User;
    type Credentials = T::Credentials;
    type Error = T::Error;

    async fn authenticate(
        &self,
        creds: Self::Credentials,
    ) -> Result<Option<Self::User>, Self::Error> {
        self.0.authenticate(creds).await
    }

    async fn get_user(
        &self,
        user_id: &axum_login::UserId<Self>,
    ) -> Result<Option<Self::User>, Self::Error> {
        self.0.get_user(user_id).await
    }
}
