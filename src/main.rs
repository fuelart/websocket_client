use fuels_core::codec::{try_from_bytes, DecoderConfig};
use tokio_tungstenite::connect_async;
use url::Url;
use futures_util::StreamExt;
use fuels::tx::Receipt;
use fuels::prelude::abigen;

abigen!(Contract(
    name = "Contract",
    abi = "abi/contract-abi.json"
));


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
                                if let Ok(data) = try_from_bytes::<CounterEvent>(&data.clone().unwrap(), DecoderConfig::default(),) {
                                    println!("contract:- {}, Log {:?}", id, data);
                                    
                                }
                            }

                    }
                    _ => {}
                }
            }
        }
    });

    // Keep the client running indefinitely
    tokio::signal::ctrl_c().await.expect("Failed to listen for Ctrl+C");

    Ok(())
}
