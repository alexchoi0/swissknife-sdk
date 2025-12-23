use rmcp::{ServiceExt, ServerHandler};
use tokio::io::{stdin, stdout};

pub async fn serve_stdio<S: ServerHandler>(service: S) -> Result<(), rmcp::Error>
where
    S: Send + 'static,
{
    let transport = (stdin(), stdout());
    let server = service.serve(transport).await?;
    server.waiting().await?;
    Ok(())
}

#[cfg(feature = "transport-sse-server")]
pub async fn serve_sse<S: ServerHandler + Clone + Send + 'static>(
    service: S,
    addr: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    use rmcp::transport::sse_server::SseServer;

    let addr: std::net::SocketAddr = addr.parse()?;
    let sse = SseServer::serve(addr).await?;

    loop {
        let transport = sse.accept().await?;
        let svc = service.clone();
        tokio::spawn(async move {
            if let Ok(server) = svc.serve(transport).await {
                let _ = server.waiting().await;
            }
        });
    }
}
