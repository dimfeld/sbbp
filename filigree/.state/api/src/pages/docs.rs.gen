use crate::server::ServerState;

pub mod _doc_id;

pub fn create_routes() -> axum::Router<ServerState> {
    axum::Router::new().merge(_doc_id::create_routes())
}
