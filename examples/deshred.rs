use chrono::Utc;
use jito_protos::shredstream::{
    shredstream_proxy_client::ShredstreamProxyClient, SubscribeEntriesRequest,
};
use std::env;
use std::str::FromStr;
use tonic::metadata::MetadataValue;
use tonic::Request;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    // Get the authentication token from environment variable
    let auth_token = env::var("SHREDSTREAM_AUTH_TOKEN")
        .expect("SHREDSTREAM_AUTH_TOKEN environment variable must be set");

    let grpc_url = env::var("SHREDSTREAM_GRPC_URL")
        .expect("SHREDSTREAM_GRPC_URL environment variable must be set");

    // Create a channel with the authentication token
    let channel = tonic::transport::Channel::from_shared(grpc_url)
        .unwrap()
        .connect()
        .await
        .unwrap();

    // Create a client with the authenticated channel
    let mut client =
        ShredstreamProxyClient::with_interceptor(channel, move |mut req: Request<()>| {
            // Add the x-token header to each request
            req.metadata_mut()
                .insert("x-token", MetadataValue::from_str(&auth_token).unwrap());
            Ok(req)
        });

    let mut stream = client
        .subscribe_entries(SubscribeEntriesRequest {})
        .await
        .unwrap()
        .into_inner();

    while let Some(slot_entry) = stream.message().await.unwrap() {
        let entries =
            match bincode::deserialize::<Vec<solana_entry::entry::Entry>>(&slot_entry.entries) {
                Ok(e) => e,
                Err(e) => {
                    println!("Deserialization failed with err: {e}");
                    continue;
                }
            };
        for entry in &entries {
            for tx in &entry.transactions {
                let account_keys = tx.message.static_account_keys();
                match account_keys.iter().find(|key| {
                    key.to_string()
                        .starts_with("Vote111111111111111111111111111111111111111")
                }) {
                    Some(_key) => {
                        continue;
                    }
                    None => {
                        println!("{} {:?}", Utc::now().to_rfc3339(), tx.signatures[0]);
                    }
                }
            }
        }
    }
    Ok(())
}
