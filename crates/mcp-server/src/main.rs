use anyhow::Result;
use clap::Parser;

#[derive(Parser)]
struct Args {
    /// Transport to use: stdio (default) or http
    #[arg(long, default_value = "stdio")]
    transport: String,
    /// HTTP port (only for http transport)
    #[arg(long, default_value = "8765")]
    port: u16,
}

fn main() -> Result<()> {
    let args = Args::parse();

    match args.transport.as_str() {
        "stdio" => run_stdio_server()?,
        _ => anyhow::bail!("Unknown transport: {}", args.transport),
    }
    Ok(())
}

fn run_stdio_server() -> Result<()> {
    // For now, just print a message that the server is ready
    // Full MCP implementation would use the mcp crate
    println!("author-clipboard-mcp ready (stdio mode)");
    Ok(())
}
