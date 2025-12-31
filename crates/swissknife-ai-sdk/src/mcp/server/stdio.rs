use rmcp::{ServiceExt, ServerHandler};
use tokio::io::{stdin, stdout};

pub async fn serve_stdio<S: ServerHandler>(service: S) -> Result<(), Box<dyn std::error::Error + Send + Sync>>
where
    S: Send + 'static,
{
    let transport = (stdin(), stdout());
    let server = service.serve(transport).await?;
    server.waiting().await?;
    Ok(())
}
