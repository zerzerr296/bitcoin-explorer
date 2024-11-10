use tokio::time::{sleep, Duration};
use mysql::*;
use mysql::prelude::*;
use reqwest;
use serde::{Serialize}; // 引入 `Serialize` 特性
use serde_json::Value; // 仅保留 `Value` 导入
use warp::Filter;
use tokio::sync::broadcast;
use anyhow::Result;
use futures_util::{StreamExt, SinkExt};
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Serialize)]
struct BlockData {
    height: i32,
    transactions: i64,
    price: f64,
    time: String,
}

async fn fetch_blockchain_data() -> Result<(i32, i64)> {
    let client = reqwest::Client::builder()
        .danger_accept_invalid_certs(true) // 忽略 SSL 验证
        .build()?;
    
    let response = client.get("https://api.blockcypher.com/v1/btc/main").send().await?;
    let json: Value = response.json().await?;
    
    println!("Received JSON: {:?}", json);
    
    let height = json["height"].as_i64().ok_or(anyhow::anyhow!("Height field missing or invalid"))?;
    let transactions = json["unconfirmed_count"].as_i64().ok_or(anyhow::anyhow!("Unconfirmed transactions field missing or invalid"))?;
    
    Ok((height as i32, transactions))
}

async fn fetch_bitcoin_price() -> Result<f64> {
    let client = reqwest::Client::builder()
        .danger_accept_invalid_certs(true) // 忽略 SSL 验证
        .build()?;
    
    let response = client
        .get("https://api.coingecko.com/api/v3/simple/price?ids=bitcoin&vs_currencies=usd")
        .send()
        .await?;
    let json: Value = response.json().await?;
    Ok(json["bitcoin"]["usd"].as_f64().unwrap())
}

async fn user_connected(ws: warp::ws::WebSocket, mut rx: tokio::sync::broadcast::Receiver<String>) {
    let (mut ws_tx, _rx) = ws.split();
    println!("A user has connected.");

    while let Ok(msg) = rx.recv().await {
        if ws_tx.send(warp::ws::Message::text(msg)).await.is_err() {
            println!("Failed to send message, closing connection.");
            break;
        }
    }
    println!("A user has disconnected.");
}

async fn get_latest_blocks(mut conn: PooledConn) -> Result<Vec<BlockData>> {
    let result: Vec<(i32, i64, f64)> = conn
        .query("SELECT block_height, transactions, price FROM blocks ORDER BY id DESC LIMIT 10")
        .unwrap();

    let data: Vec<BlockData> = result
        .into_iter()
        .map(|(height, transactions, price)| BlockData {
            height,
            transactions,
            price,
            time: chrono::Utc::now().to_rfc3339(), // 使用 UTC 时间戳
        })
        .collect();

    Ok(data)
}

#[tokio::main]
async fn main() {
    let tx = Arc::new(Mutex::new(broadcast::channel(100).0)); // 使用 Arc<Mutex<>> 包装 tx

    let websocket_route = warp::path("ws")
        .and(warp::ws())
        .map({
            let tx = Arc::clone(&tx); // 克隆 Arc
            move |ws: warp::ws::Ws| {
                let tx = Arc::clone(&tx); // 再次克隆 Arc
                ws.on_upgrade(move |socket| {
                    let tx = Arc::clone(&tx);
                    async move {
                        let rx = {
                            let tx = tx.lock().await;
                            tx.subscribe()
                        };
                        user_connected(socket, rx).await;
                    }
                })
            }
        });

    let url = "mysql://zerzerr917:Ywy20010917.@mysql:3306/bitcoin_explorer";
    let pool = loop {
        match Pool::new(url) {
            Ok(pool) => break pool,
            Err(_) => {
                println!("Failed to connect to MySQL, retrying...");
                sleep(Duration::from_secs(5)).await;
            }
        };
    };

    let latest_blocks_route = warp::path("latest_blocks")
        .and(warp::get())
        .and_then({
            let pool = pool.clone();
            move || {
                let conn = pool.get_conn().unwrap();
                async move {
                    let blocks = get_latest_blocks(conn).await.unwrap(); // 使用 async 和 await
                    Ok::<_, warp::Rejection>(warp::reply::json(&blocks)) // 返回 JSON 响应
                }
            }
        });

    let routes = websocket_route
        .or(latest_blocks_route);

    let mut conn = pool.get_conn().unwrap();
    let tx_clone = Arc::clone(&tx); // 克隆 Arc

    tokio::spawn(async move {
        loop {
            match fetch_blockchain_data().await {
                Ok((height, transactions)) => {
                    conn.exec_drop(
                        "INSERT INTO blocks (block_height, transactions, price) VALUES (:height, :transactions, :price)",
                        params! {
                            "height" => height,
                            "transactions" => transactions,
                            "price" => fetch_bitcoin_price().await.unwrap(),
                        }
                    ).unwrap();

                    let price = fetch_bitcoin_price().await.unwrap();
                    let message = format!(
                        r#"{{"height": {}, "transactions": {}, "price": {}}}"#,
                        height, transactions, price
                    );

                    println!("Attempting to broadcast message: {}", message);
                    // 广播数据给所有连接的客户端
                    if let Err(e) = {
                        let tx = tx_clone.lock().await;
                        tx.send(message.clone())
                    } {
                        println!("Failed to broadcast message: {}", e);
                    } else {
                        println!("Broadcasted message: {}", message);
                    }

                    conn.exec_drop(
                        "INSERT INTO blocks (block_height, transactions, price) VALUES (:height, :transactions, :price)",
                        params! {
                            "height" => height,
                            "transactions" => transactions,
                            "price" => price,
                        }
                    ).unwrap();
                }
                Err(e) => {
                    println!("Failed to fetch blockchain data: {}", e);
                }
            }

            sleep(Duration::from_secs(60)).await; // 每 60 秒获取一次数据
        }
    });

    println!("WebSocket server started at ws://0.0.0.0:3030/ws");

    warp::serve(routes).run(([0, 0, 0, 0], 3030)).await;
}
