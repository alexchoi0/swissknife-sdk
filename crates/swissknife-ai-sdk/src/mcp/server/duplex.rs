use rmcp::{
    service::{RoleClient, Peer, RunningService, ServiceExt},
    ServerHandler,
};

type ClientService = RunningService<RoleClient, ()>;

pub struct DuplexConnection {
    pub server_handle: tokio::task::JoinHandle<()>,
    pub peer: Peer<RoleClient>,
}

pub async fn serve_duplex<S: ServerHandler + Send + 'static>(
    server: S,
) -> Result<DuplexConnection, Box<dyn std::error::Error + Send + Sync>> {
    let (server_transport, client_transport) = tokio::io::duplex(4096);

    let server_handle = tokio::spawn(async move {
        match server.serve(server_transport).await {
            Ok(running) => {
                if let Err(e) = running.waiting().await {
                    eprintln!("MCP server error: {:?}", e);
                }
            }
            Err(e) => eprintln!("MCP server init error: {:?}", e),
        }
    });

    let client_service: ClientService = ().serve(client_transport).await?;
    let peer = client_service.peer().clone();

    Ok(DuplexConnection {
        server_handle,
        peer,
    })
}
