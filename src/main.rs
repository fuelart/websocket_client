use fuels::prelude::abigen;
use fuels::tx::Receipt;
use fuels_core::codec::{try_from_bytes, DecoderConfig};
use futures_util::StreamExt;
use tokio_tungstenite::connect_async;
use url::Url;

use hyperfuel_client::{Client, Config};
use std::num::NonZeroU64;
use tokio::time::{sleep, Duration};

abigen!(Contract(name = "Contract", abi = "abi/contract-abi.json"));

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let contract_id = "627bd0b97b8b6c129b36d41c7518c868ca1cf972cb735c93a2fe088b39cb668d".to_owned();

    // Replace with your WebSocket server URL
    // let url = Url::parse("ws://localhost:8080")?;
    let url = Url::parse("ws://20.55.16.129:8080")?;
    let (ws_stream, _) = connect_async(url.clone()).await?;
    let (_, mut read) = ws_stream.split();
    println!("WebSocket is connected on: {}", url);

    tokio::spawn(async move {
        while let Some(message) = read.next().await {
            let message = message.unwrap();
            if message.is_binary() {
                // Deserialize and handle the binary message here
                let rcpts: Receipt = bincode::deserialize(&message.into_data()).unwrap();
                //  println!("Received a receipt: {:?}", rcpts);
                #[allow(unused)]
                match rcpts {
                    Receipt::LogData {
                        id,
                        ra,
                        rb,
                        ptr,
                        len,
                        digest,
                        data,
                        pc,
                        is,
                    } => {
                        if *id.to_string() == contract_id {
                            if let Ok(data) = try_from_bytes::<CounterEvent>(
                                &data.clone().unwrap(),
                                DecoderConfig::default(),
                            ) {
                                println!("WebSocket: contract:- {}, Log {:?}", id, data);
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    });

    // Hypersync polling
    let client_config = Config {
        url: Url::parse("https://fuel-testnet.hypersync.xyz").unwrap(),
        bearer_token: None,
        http_req_timeout_millis: NonZeroU64::new(30000).unwrap(),
    };
    let client = Client::new(client_config).unwrap();

    let contracts = vec![hex_literal::hex!(
        "627bd0b97b8b6c129b36d41c7518c868ca1cf972cb735c93a2fe088b39cb668d"
    )];
    let mut from_block = client.get_height().await.unwrap();

    tokio::spawn(async move {
        loop {
            // Here we are doing a preset query, but you can construct your own more advanced queries using this schema.
            let logs = client
                .preset_query_get_logs(contracts.clone(), from_block, None)
                .await
                .unwrap();

            logs.data.into_iter().for_each(|log| {
                if let Ok(data) = try_from_bytes::<CounterEvent>(
                    &log.data.clone().unwrap(),
                    DecoderConfig::default(),
                ) {
                    println!("HYPERSYNC: Log {:?}", data);
                }
            });

            from_block = logs.next_block;

            // A polling interval of 400ms is reasonable for getting fresh data fast.
            sleep(Duration::from_millis(400)).await;
        }
    });

    // Keep the client running indefinitely
    tokio::signal::ctrl_c()
        .await
        .expect("Failed to listen for Ctrl+C");

    Ok(())
}
