use tokio::time::{sleep, Duration};
use mysql::*;
use mysql::prelude::*;
use reqwest;
use serde::{Serialize};
use serde_json::Value;
use warp::Filter;
use tokio::sync::broadcast;
use anyhow::Result;
use futures_util::{StreamExt, SinkExt};
use std::sync::Arc;
use tokio::sync::Mutex;
use chrono::{DateTime, Utc}; // 引入 chrono 库

#[derive(Serialize)]
struct BlockData {
    height: i32,
    transactions: i64,
    price: f64,
    time: String, // 添加时间字段
}

// 将 API 时间格式转换为 MySQL 可接受的格式
fn convert_to_mysql_timestamp(iso_time: &str) -> String {
    let datetime: DateTime<Utc> = DateTime::parse_from_rfc3339(iso_time)
        .expect("Failed to parse datetime")
        .with_timezone(&Utc);
    datetime.format("%Y-%m-%d %H:%M:%S").to_string() // 返回 MySQL 格式的时间
}
// 将数据库时间戳转换为符合 WebSocket 数据格式的字符串
fn convert_to_websocket_format(timestamp: &str) -> String {
    // 使用 chrono 库转换时间
    let datetime: DateTime<Utc> = DateTime::parse_from_str(timestamp, "%Y-%m-%d %H:%M:%S")
        .expect("Failed to parse timestamp")
        .with_timezone(&Utc);

    // 转换为 WebSocket 格式（ISO 8601 格式）
    datetime.to_rfc3339()  // 返回 ISO 8601 格式的字符串
}

async fn fetch_blockchain_data() -> Result<(i32, i64, String, String)> {
    let client = reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .build()?;
    
    let response = client.get("https://api.blockcypher.com/v1/btc/main").send().await?;
    let json: Value = response.json().await?;
    
    println!("Received JSON: {:?}", json);
    
    let height = json["height"].as_i64().ok_or(anyhow::anyhow!("Height field missing or invalid"))? as i32;
    let transactions = json["unconfirmed_count"].as_i64().ok_or(anyhow::anyhow!("Unconfirmed transactions field missing or invalid"))?;
    let hash = json["hash"].as_str().ok_or(anyhow::anyhow!("Hash field missing or invalid"))?.to_string();
    let time = json["time"].as_str().ok_or(anyhow::anyhow!("Time field missing or invalid"))?.to_string(); // 获取时间字段
    let mysql_time = convert_to_mysql_timestamp(&time); // 转换为 MySQL 可接受的时间格式
    
    Ok((height, transactions, hash, mysql_time)) // 返回转换后的时间字段
}

async fn fetch_bitcoin_price() -> Result<f64> {
    let client = reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
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
            break;
        }
    }
    println!("A user has disconnected.");
}

async fn get_latest_blocks(mut conn: PooledConn) -> Result<Vec<BlockData>> {
    let result: Vec<(i32, i64, f64, String)> = conn
        .query("SELECT block_height, transactions, price, timestamp FROM blocks ORDER BY id DESC LIMIT 10")
        .unwrap();

    let data: Vec<BlockData> = result
        .into_iter()
        .map(|(height, transactions, price, timestamp)| BlockData {
            height,
            transactions,
            price,
            time: convert_to_websocket_format(&timestamp), // 使用数据库中存储的时间
        })
        .collect();

    Ok(data)
}

#[tokio::main]
async fn main() {
    let tx = Arc::new(Mutex::new(broadcast::channel(100).0)); // 使用 Arc<Mutex<>> 包装 tx

    // WebSocket 路由
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

    // MySQL 连接池
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

    // 获取最新的区块数据
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

    // CORS 配置
    let cors = warp::cors()
        .allow_any_origin()  // 允许任何来源
        .allow_methods(vec!["GET", "POST"]) // 允许GET和POST请求
        .allow_headers(vec!["Content-Type", "Authorization"]);

    // 配置CORS到路由中
    let routes = websocket_route
        .or(latest_blocks_route)
        .with(cors);  // 使用 CORS 策略

    let mut conn = pool.get_conn().unwrap();
    let tx_clone = Arc::clone(&tx); // 克隆 Arc

    tokio::spawn(async move {
        loop {
            match fetch_blockchain_data().await {
                Ok((height, transactions, hash, time)) => { // 获取到时间字段
                    // 确保所有数据已获取后，再获取价格
                    if let Ok(price) = fetch_bitcoin_price().await {
                        conn.exec_drop(
                            "INSERT INTO blocks (block_height, transactions, price, hash, timestamp) VALUES (:height, :transactions, :price, :hash, :timestamp)",
                            params! {
                                "height" => height,
                                "transactions" => transactions,
                                "price" => price,
                                "hash" => hash,
                                "timestamp" => &time, // 使用转换后的时间
                            }
                        ).unwrap();

                        let message = format!(
                            r#"{{"height": {}, "transactions": {}, "price": {}, "timestamp": "{}"}}"#,
                            height, transactions, price, time
                        );

                        // 广播数据给所有连接的客户端
                        if let Err(e) = {
                            let tx = tx_clone.lock().await;
                            tx.send(message.clone())
                        } {
                            println!("Failed to broadcast message: {}", e);
                        } else {
                            println!("Broadcasted message: {}", message);
                        }

                        println!("Inserted: Height: {}, Transactions: {}, Price: {}, Timestamp: {}", height, transactions, price, time);
                    } else {
                        println!("Failed to fetch price, skipping insertion");
                    }
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
