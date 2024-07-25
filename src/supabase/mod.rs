mod auth;
mod error;
mod session_store;
mod user_identity;
mod wrappers;

use std::sync::{Arc, Weak};

use tower_sessions_moka_store::MokaStore;
use user_identity::IdentityData;
use wrappers::{AuthWrapper, StoreWrapper};

pub use error::SupabaseError;

pub type AuthSession = axum_login::AuthSession<AuthWrapper<SupabaseBackend>>;
pub type Supabase = std::sync::Arc<SupabaseBackend>;

#[derive(Debug, Clone)]
pub struct AppUser {
    pub identity: IdentityData,
}

#[derive(Clone, Debug)]
pub struct SupabaseBackend {
    pub client: supabase_rust::Supabase,
    weak: Weak<Self>,
    sessions_cache: MokaStore,
    service_key: String,
}

impl SupabaseBackend {
    pub fn new(sessions_cache: MokaStore) -> Arc<Self> {
        let client = supabase_rust::Supabase::new(None, None, None);
        let service_key =
            std::env::var("SUPABASE_SERVICE_KEY").expect("env var SUPABASE_SERVICE_KEY not set");
        Arc::new_cyclic(|this: &Weak<Self>| Self {
            client,
            weak: this.clone(),
            sessions_cache,
            service_key,
        })
    }

    pub fn as_auth_backend(&self) -> AuthWrapper<Self> {
        AuthWrapper(Arc::clone(&self.weak.upgrade().unwrap()))
    }

    pub fn as_session_store(&self) -> StoreWrapper<Self> {
        StoreWrapper(Arc::clone(&self.weak.upgrade().unwrap()))
    }
}
