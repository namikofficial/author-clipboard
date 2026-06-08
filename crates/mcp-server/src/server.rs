use author_clipboard_shared::ipc::{IpcClient, IpcCommand};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchInput {
    pub query: String,
    pub limit: Option<usize>,
}

pub struct McpServer {
    client: IpcClient,
}

impl McpServer {
    pub fn new() -> Self {
        Self {
            client: IpcClient::new(),
        }
    }

    pub fn search(&self, input: SearchInput) -> Result<serde_json::Value, String> {
        let response = self
            .client
            .send_command(&IpcCommand::Search {
                query: input.query,
                limit: input.limit,
                filters: None,
            })
            .map_err(|e| e.to_string())?;

        response.data.ok_or_else(|| "No data".to_string())
    }
}

impl Default for McpServer {
    fn default() -> Self {
        Self::new()
    }
}
