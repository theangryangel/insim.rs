use crate::config::web::WebConfig;
use axum::Router;

pub(crate) fn start(config: &WebConfig) {
    let app = Router::new();

    tracing::info!("Web listening on {:?}", &config.address);

    tokio::task::spawn(axum::Server::bind(&config.address.unwrap()).serve(app.into_make_service()));
}
