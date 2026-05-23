use kimi_wire::transport::{ChildProcessTransport, TransportWireClient};
use kimi_wire::WireClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let transport = ChildProcessTransport::spawn("kimi", None, None, None).await?;
    let mut client = TransportWireClient::new(transport);

    let result = client.prompt("Refactor this code").await?;
    println!("Turn finished with status: {:?}", result.status);

    Ok(())
}
