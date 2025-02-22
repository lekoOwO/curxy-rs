use anyhow::Result;
use clap::Parser;
use libcfd::{
    connection::{ConnectResponse, Connection, DstAddr, SrcAddr},
    tunnel_config::TunnelConfig,
};
use std::collections::HashMap;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::task::spawn_local;
use url::Url;

mod proxy;
mod util;

#[derive(Parser, Debug)]
#[command(name = "curxy")]
#[command(about = "A proxy worker for using ollama in cursor")]
struct Args {
    #[arg(
        short = 'e',
        long,
        default_value = "http://localhost:11434",
        value_parser = validate_url,
        help = "The endpoint to Ollama server"
    )]
    endpoint: String,

    #[arg(
        short = 'o',
        long,
        default_value = "https://api.openai.com",
        value_parser = validate_url,
        help = "The endpoint to OpenAI server"
    )]
    openai_endpoint: String,

    #[arg(
        short = 'p',
        long,
        help = "The port to run the server on. Default is random"
    )]
    port: Option<u16>,

    #[arg(
        long,
        default_value = "127.0.0.1",
        help = "The hostname to run the server on"
    )]
    hostname: String,

    #[arg(
        long,
        default_value_t = true,
        help = "Use cloudflared to tunnel the server"
    )]
    cloudflared: bool,
}

fn validate_url(url: &str) -> Result<String, String> {
    match Url::parse(url) {
        Ok(_) => Ok(url.to_string()),
        Err(_) => Err("Invalid URL".to_string()),
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let openai_api_key = std::env::var("OPENAI_API_KEY").ok();

    let app = proxy::create_app(
        &args.openai_endpoint,
        &args.endpoint,
        openai_api_key.as_deref(),
    );

    let port = args.port.unwrap_or(util::get_random_port().await?);
    let addr = format!("{}:{}", args.hostname, port);

    let local = tokio::task::LocalSet::new();

    local
        .run_until(async move {
            let cloudflared_handle = if args.cloudflared {
                let config = TunnelConfig::try_cloudflare().await.unwrap();
                println!("Server running at: https://{}", config.hostname);
                println!(
                    "Enter https://{}/v1 into Override OpenAI Base URL section in cursor settings",
                    config.hostname
                );

                let (connection, connection_details) =
                    Connection::new(config, port as u8, SrcAddr::Default, DstAddr::Default)
                        .await
                        .unwrap();
                println!("Registered tunnel connection: {:#?}", connection_details);

                Some(spawn_local(async move {
                    loop {
                        let connect_request = connection.accept().await.unwrap();
                        spawn_local(async move {
                            if let Err(error) = async {
                                let mut metadata = HashMap::new();
                                metadata.insert("HttpStatus".to_string(), "200".to_string());

                                metadata.insert(
                                    "HttpHeader:Content-Type".to_string(),
                                    "application/json".to_string(),
                                );
                                metadata.insert(
                                    "HttpHeader:Access-Control-Allow-Origin".to_string(),
                                    "*".to_string(),
                                );
                                metadata.insert(
                                    "HttpHeader:Access-Control-Allow-Methods".to_string(),
                                    "GET, POST, OPTIONS".to_string(),
                                );
                                metadata.insert(
                                    "HttpHeader:Access-Control-Allow-Headers".to_string(),
                                    "Content-Type, Authorization".to_string(),
                                );

                                let (mut send_stream, mut recv_stream) = connect_request
                                    .respond_with(ConnectResponse::Metadata(metadata))
                                    .await?;

                                let mut tcp =
                                    tokio::net::TcpStream::connect(("127.0.0.1", port)).await?;
                                let mut read_buffer = vec![0; 8192];
                                let mut write_buffer = vec![0; 8192];

                                loop {
                                    tokio::select! {
                                        n = recv_stream.read(&mut read_buffer) => {
                                            match n? {
                                                Some(0) | None => break,
                                                Some(n) => tcp.write_all(&read_buffer[..n]).await?,
                                            }
                                        }
                                        n = tcp.read(&mut write_buffer) => {
                                            let n = n?;
                                            if n == 0 { break; }
                                            send_stream.write_all(&write_buffer[..n]).await?;
                                        }
                                    }
                                }

                                Ok::<_, anyhow::Error>(())
                            }
                            .await
                            {
                                eprintln!("Tunnel error: {}", error);
                            }
                        });
                    }
                }))
            } else {
                None
            };

            let server_handle = spawn_local(async move {
                axum::serve(
                    tokio::net::TcpListener::bind(&addr).await?,
                    app.into_make_service(),
                )
                .await
            });

            tokio::select! {
                res = async {
                    if let Some(handle) = cloudflared_handle {
                        handle.await.map_err(|e| anyhow::anyhow!("Cloudflared error: {}", e))?;
                    }
                    Ok::<_, anyhow::Error>(())
                } => { let _ = res; },
                result = server_handle => {
                    let _ = result.map_err(|e| anyhow::anyhow!("Server error: {}", e))
                        .and_then(|r| r.map_err(|e| anyhow::anyhow!("HTTP error: {}", e)));
                }
            }
        })
        .await;

    Ok(())
}
