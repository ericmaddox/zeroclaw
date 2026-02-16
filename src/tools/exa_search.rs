use super::traits::{Tool, ToolResult};
use async_trait::async_trait;
use serde_json::json;

pub struct ExaSearchTool {
    api_key: String,
}

impl ExaSearchTool {
    pub fn new(api_key: String) -> Self {
        Self { api_key }
    }
}

#[async_trait]
impl Tool for ExaSearchTool {
    fn name(&self) -> &str {
        "exa_search"
    }

    fn description(&self) -> &str {
        "Search the web using Exa AI for real-time information and high-quality links"
    }

    fn parameters_schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "query": {
                    "type": "string",
                    "description": "The search query"
                },
                "num_results": {
                    "type": "integer",
                    "description": "Number of results to return (default 5)",
                    "default": 5
                },
                "use_autoprompt": {
                    "type": "boolean",
                    "description": "Whether to use Exa's autoprompt feature",
                    "default": true
                }
            },
            "required": ["query"]
        })
    }

    async fn execute(&self, args: serde_json::Value) -> anyhow::Result<ToolResult> {
        let query = args.get("query").and_then(|v| v.as_str()).unwrap_or("");
        let num_results = args.get("num_results").and_then(|v| v.as_u64()).unwrap_or(5);
        let use_autoprompt = args.get("use_autoprompt").and_then(|v| v.as_bool()).unwrap_or(true);

        if self.api_key.is_empty() {
            return Ok(ToolResult {
                success: false,
                output: String::new(),
                error: Some("Exa API key is not configured. Please add 'exa_api_key' to [search] in your config.toml.".to_string()),
            });
        }

        let client = reqwest::Client::new();
        let resp = client
            .post("https://api.exa.ai/search")
            .header("x-api-key", &self.api_key)
            .header("Content-Type", "application/json")
            .json(&json!({
                "query": query,
                "numResults": num_results,
                "useAutoprompt": use_autoprompt,
            }))
            .send()
            .await?;

        if !resp.status().is_success() {
            let error_text = resp.text().await?;
            return Ok(ToolResult {
                success: false,
                output: String::new(),
                error: Some(format!("Exa AI API error ({}): {}", resp.status(), error_text)),
            });
        }

        let body: serde_json::Value = resp.json().await?;
        Ok(ToolResult {
            success: true,
            output: serde_json::to_string_pretty(&body)?,
            error: None,
        })
    }
}
