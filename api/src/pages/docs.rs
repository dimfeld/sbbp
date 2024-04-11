use crate::server::ServerState;

pub mod _docId;

pub fn create_routes() -> axum::Router<ServerState> {
    axum::Router::new().merge(_docId::create_routes())
}
