use kimi_wire::protocol::{PromptStatus, RawWireMessage};
use kimi_wire::{InMemoryWireClient, WireClient};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = InMemoryWireClient::new();

    // Inject a mock response.
    client
        .inject(RawWireMessage {
            jsonrpc: kimi_wire::protocol::JsonRpcVersion::V2,
            id: Some("req-1".to_string()),
            method: None,
            params: None,
            result: Some(serde_json::json!({"status": "finished"})),
            error: None,
        })
        .await;

    let result = client.prompt("Hello!").await?;
    assert_eq!(result.status, PromptStatus::Finished);

    println!("Mock test passed!");
    Ok(())
}
