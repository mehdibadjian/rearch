use anyhow::{Context, Result};
use rearch_core::guard::scan_untrusted_output;
use serde_json::{json, Value};
use std::path::Path;
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, ChildStdin, ChildStdout, Command};

pub struct ReaMcpClient {
    _child: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
    next_id: u64,
}

impl ReaMcpClient {
    pub async fn spawn() -> Result<Self> {
        Self::spawn_cmd("npx", &["-y", "rea-agents@latest", "mcp"]).await
    }

    pub async fn spawn_cmd(cmd: &str, args: &[&str]) -> Result<Self> {
        let mut child = Command::new(cmd)
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .with_context(|| format!("Failed to spawn MCP server via {} {:?}", cmd, args))?;

        let stdin = child.stdin.take().context("Missing child stdin")?;
        let stdout = BufReader::new(child.stdout.take().context("Missing child stdout")?);

        Ok(Self {
            _child: child,
            stdin,
            stdout,
            next_id: 1,
        })
    }

    pub async fn call_tool_at(
        &mut self,
        base_dir: &Path,
        slug: &str,
        tool_name: &str,
        arguments: Value,
    ) -> Result<Value> {
        let req_id = self.next_id;
        self.next_id += 1;

        let payload = json!({
            "jsonrpc": "2.0",
            "id": req_id,
            "method": "tools/call",
            "params": {
                "name": tool_name,
                "arguments": arguments
            }
        });

        let mut line = serde_json::to_string(&payload)?;
        line.push('\n');
        self.stdin.write_all(line.as_bytes()).await?;
        self.stdin.flush().await?;

        let mut resp_line = String::new();
        self.stdout.read_line(&mut resp_line).await?;
        let resp: Value = serde_json::from_str(&resp_line)
            .with_context(|| format!("Failed to parse JSON-RPC response: {}", resp_line))?;

        let raw_text = resp.to_string();
        scan_untrusted_output(base_dir, slug, tool_name, &raw_text)?;

        Ok(resp["result"].clone())
    }

    pub async fn call_tool(
        &mut self,
        slug: &str,
        tool_name: &str,
        arguments: Value,
    ) -> Result<Value> {
        self.call_tool_at(Path::new("."), slug, tool_name, arguments).await
    }
}
