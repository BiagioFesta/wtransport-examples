use anyhow::Result;
use axum::routing::get;
use axum::Router;
use axum::Server;
use std::net::Ipv6Addr;
use std::net::SocketAddr;
use std::path::Path;
use tower_http::services::ServeDir;

pub const HTTP_PORT: u16 = 8080;

pub async fn http_server<P>(directory: P, fingerprint: &str) -> Result<()>
where
    P: AsRef<Path>,
{
    let routes = Router::new()
        .nest_service("/", ServeDir::new(directory))
        .route(
            "/fingerprint",
            get({
                let fingerprint = fingerprint.to_string();
                || async { fingerprint }
            }),
        );

    Server::bind(&SocketAddr::new(Ipv6Addr::UNSPECIFIED.into(), HTTP_PORT))
        .serve(routes.into_make_service())
        .await?;

    Ok(())
}
