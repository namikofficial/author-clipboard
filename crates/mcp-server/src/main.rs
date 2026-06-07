use anyhow::Result;
use author_clipboard_mcp::ClipboardService;
use clap::Parser;
use mcp_server::{ByteTransport, Server};
use tokio::io::{stdin, stdout, BufReader};

#[derive(Parser)]
struct Args {
    #[arg(long, default_value = "stdio")]
    transport: String,
    #[arg(long, default_value = "8765")]
    port: u16,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    match args.transport.as_str() {
        "stdio" => run_stdio_server().await?,
        _ => anyhow::bail!("Unknown transport: {}", args.transport),
    }
    Ok(())
}

async fn run_stdio_server() -> Result<()> {
    let service = ClipboardService::new();
    let server = Server::new(service);

    let transport = ByteTransport::new(BufReader::new(stdin()), stdout());

    server.run(transport).await?;
    Ok(())
}