use axum::body::Body;
use http::{Request, Response, StatusCode};

use crate::supabase::AuthSession;

pub async fn require_login(req: Request<Body>) -> Result<Request<Body>, Response<Body>> {
    let Some(auth_session) = req.extensions().get::<AuthSession>() else {
        return Err(Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(Body::empty())
            .unwrap());
    };

    if auth_session.user.is_none() {
        return Err(Response::builder().body(Body::empty()).unwrap());
    }

    Ok(req)
}
