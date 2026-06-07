//! author-clipboard MCP server
//!
//! A Model Context Protocol server that exposes clipboard history
//! as tools, resources, and prompts for AI coding agents.

mod error;
mod resources;
mod server;
mod tools;

pub use error::McpError;
pub use server::McpServer;
