mod command;
mod db;
mod resp;

use bytes::BytesMut;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use log::{info, error};
use tokio::sync::broadcast;

use crate::command::Command;
use crate::db::Db;
use crate::resp::Frame;

async fn run_server(port: u16, shutdown: Option<broadcast::Receiver<()>>) -> Result<(), Box<dyn std::error::Error>> {
    let db = Arc::new(Db::new());
    let addr = format!("127.0.0.1:{}", port);
    let listener = TcpListener::bind(&addr).await?;
    info!("Server listening on {}", addr);

    let mut shutdown_rx = shutdown.unwrap_or_else(|| {
        let (_, rx) = broadcast::channel(1);
        rx
    });

    loop {
        tokio::select! {
            accept_result = listener.accept() => {
                match accept_result {
                    Ok((socket, addr)) => {
                        info!("Accepted connection from: {}", addr);
                        let db = db.clone();
                        tokio::spawn(async move {
                            if let Err(e) = process_client(socket, db).await {
                                error!("Error processing client: {}", e);
                            }
                        });
                    }
                    Err(e) => {
                        error!("Failed to accept connection: {}", e);
                    }
                }
            }
            _ = shutdown_rx.recv() => {
                info!("Shutdown signal received, stopping server...");
                break;
            }
        }
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    run_server(6379, None).await
}

async fn process_client(mut socket: TcpStream, db: Arc<Db>) -> Result<(), Box<dyn std::error::Error>> {
    let mut buffer = BytesMut::with_capacity(4096);

    loop {
        if 0 == socket.read_buf(&mut buffer).await? {
            return Ok(());
        }

        match Frame::parse(&mut buffer) {
            Ok(Some(frame)) => {
                match Command::from_frame(frame) {
                    Ok(cmd) => {
                        let response = cmd.execute(&db);
                        socket.write_all(&response.encode()).await?;
                    }
                    Err(e) => {
                        let error = Frame::Error(e);
                        socket.write_all(&error.encode()).await?;
                    }
                }
            }
            Ok(None) => continue,
            Err(e) => {
                let error = Frame::Error(e.to_string());
                socket.write_all(&error.encode()).await?;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use redis::Client;
    use std::time::Duration;
    use tokio::time::timeout;

    const TEST_PORT: u16 = 6380; // Use a different port for testing

    #[tokio::test]
    async fn test_redis_integration() {
        // Setup shutdown channel
        let (shutdown_tx, shutdown_rx) = broadcast::channel(1);

        // Start server in a separate tokio runtime
        let server_handle = tokio::spawn(async move {
            if let Err(e) = run_server(TEST_PORT, Some(shutdown_rx)).await {
                eprintln!("Server error: {}", e);
            }
        });

        // Wait for server to start and try to connect
        let mut client = None;
        for _ in 0..3 {
            tokio::time::sleep(Duration::from_millis(500)).await;
            match Client::open(format!("redis://127.0.0.1:{}", TEST_PORT)) {
                Ok(c) => {
                    client = Some(c);
                    break;
                }
                Err(_) => continue,
            }
        }

        let client = client.expect("Failed to connect to test server");
        let mut con = client.get_connection().unwrap();

        // Run tests with timeout
        let test_result = timeout(Duration::from_secs(5), async {
            // Test SET and GET
            let _: () = redis::cmd("SET")
                .arg("test_key")
                .arg("test_value")
                .query(&mut con)
                .unwrap();

            let value: String = redis::cmd("GET")
                .arg("test_key")
                .query(&mut con)
                .unwrap();

            assert_eq!(value, "test_value");

            // Test DEL
            let deleted: i32 = redis::cmd("DEL")
                .arg("test_key")
                .query(&mut con)
                .unwrap();

            assert_eq!(deleted, 1);
        }).await;

        // Signal server to shut down
        let _ = shutdown_tx.send(());

        // Wait for server to shut down with timeout
        let _ = timeout(Duration::from_secs(1), server_handle).await;

        // Drop the connection explicitly
        drop(con);
        drop(client);

        // Assert that the test completed within the timeout
        assert!(test_result.is_ok(), "Test timed out");
    }
} 