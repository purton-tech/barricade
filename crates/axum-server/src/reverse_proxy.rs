use axum::{
    body::{Body},
    http::{Request},
    response::{IntoResponse, Response},
};
use tokio::net::TcpStream;
use hyper::client::conn::Builder;

pub async fn proxy(req: Request<Body>) -> Result<Response, hyper::Error> {
    tracing::trace!("Forwarding: {:?}", req.uri());

    let addr =  "whoami:80".to_string();

    let stream = TcpStream::connect(addr).await.unwrap();

    let (mut sender, conn) = Builder::new()
        .http1_preserve_header_case(true)
        .http1_title_case_headers(true)
        .handshake(stream)
        .await?;

    tokio::task::spawn(async move {
        if let Err(err) = conn.await {
            tracing::error!("Connection failed: {:?}", err);
        }
    });

    let resp = sender.send_request(req).await?;
    Ok(resp.into_response())
}
