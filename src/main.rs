use anyhow::Result;
use clap::Parser;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command as AsyncCommand;
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

    #[arg(long, help = "Path to cloudflared executable")]
    cloudflared_path: Option<String>,
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

    println!("Starting server on {}", addr);

    // 啟動 HTTP 服務器
    let server_handle = tokio::spawn(async move {
        axum::serve(
            tokio::net::TcpListener::bind(&addr).await?,
            app.into_make_service(),
        )
        .await
    });

    // 如果指定了 cloudflared，則啟動 tunnel
    if let Some(cloudflared) = args.cloudflared_path {
        let tunnel_handle = tokio::spawn(async move {
            let output = AsyncCommand::new(cloudflared)
                .arg("tunnel")
                .arg("--url")
                .arg(format!("http://localhost:{}", port))
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::piped())
                .spawn()?;

            let mut child = output;

            // 讀取並印出 stdout
            if let Some(stdout) = child.stdout.take() {
                let reader = BufReader::new(stdout);
                let mut lines = reader.lines();

                while let Some(line) = lines.next_line().await? {
                    print!("{}\n", line);
                }
            }

            child.wait().await?;
            Ok::<_, anyhow::Error>(())
        });

        tokio::select! {
            res = tunnel_handle => {
                if let Err(e) = res? {
                    eprintln!("Cloudflared error: {}", e);
                }
            }
            res = server_handle => {
                if let Err(e) = res? {
                    eprintln!("Server error: {}", e);
                }
            }
        }
    } else {
        if let Err(e) = server_handle.await? {
            eprintln!("Server error: {}", e);
        }
    }

    Ok(())
}
