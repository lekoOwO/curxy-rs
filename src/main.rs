use anyhow::Result;
use clap::Parser;
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

    if args.cloudflared {
        // 啟動 cloudflared 通道
        tokio::spawn(async move {
            let tunnel = cloudflared::Tunnel::builder()
                .url(format!("http://{}:{}", args.hostname, port).as_str())
                .build()
                .unwrap();

            println!("Server running at: {}", tunnel.url());
            println!(
                "Enter {}/v1 into Override OpenAI Base URL section in cursor settings",
                tunnel.url()
            );
        });
    }

    println!("Starting server on {}", addr);
    axum::serve(
        tokio::net::TcpListener::bind(&addr).await?,
        app.into_make_service(),
    )
    .await?;

    Ok(())
}
