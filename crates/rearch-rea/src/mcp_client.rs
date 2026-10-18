use anyhow::{Context, Result};
use goblin::Object;
use rearch_core::gates::verify_inventory_populated_at;
use rearch_core::guard::scan_untrusted_output;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::fs;
use std::path::Path;
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, ChildStdin, ChildStdout, Command};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReaSessionRecord {
    pub engine: String,
    pub target_sha256: String,
}

/// Enforces REA Law 1 (Reuse Before Re-Inspecting):
/// Checks `analysis/<slug>/rea_sessions.json` before opening a duplicate Ghidra/JADX/CDP session
/// if one is already indexed for the target SHA-256 digest.
/// Returns `Ok(true)` if an existing session was found and reused, `Ok(false)` if newly registered.
pub fn check_or_register_session(
    base_dir: &Path,
    slug: &str,
    engine: &str,
    target_sha256: &str,
) -> Result<bool> {
    let analysis_dir = base_dir.join("analysis").join(slug);
    fs::create_dir_all(&analysis_dir)?;
    let sessions_path = analysis_dir.join("rea_sessions.json");

    let mut sessions: Vec<ReaSessionRecord> = if sessions_path.exists() {
        serde_json::from_str(&fs::read_to_string(&sessions_path)?).unwrap_or_default()
    } else {
        Vec::new()
    };

    if sessions
        .iter()
        .any(|s| s.engine == engine && s.target_sha256 == target_sha256)
    {
        return Ok(true);
    }

    sessions.push(ReaSessionRecord {
        engine: engine.to_string(),
        target_sha256: target_sha256.to_string(),
    });
    fs::write(&sessions_path, serde_json::to_string_pretty(&sessions)?)?;
    Ok(false)
}

/// Format-aware native binary inventory using `goblin` (ELF/PE/Mach-O/Archive).
pub fn inspect_binary_inventory(bytes: &[u8]) -> Result<Value> {
    match Object::parse(bytes) {
        Ok(Object::Elf(elf)) => {
            let symbols: Vec<String> = elf
                .dynsyms
                .iter()
                .filter_map(|s| elf.dynstrtab.get_at(s.st_name).map(|n| n.to_string()))
                .filter(|n| !n.is_empty())
                .collect();
            Ok(json!({
                "format": "ELF",
                "is_64": elf.is_64,
                "entry": elf.entry,
                "libraries": elf.libraries,
                "exported_symbols": symbols,
            }))
        }
        Ok(Object::PE(pe)) => {
            let exports: Vec<String> = pe
                .exports
                .iter()
                .filter_map(|e| e.name.map(|n| n.to_string()))
                .collect();
            Ok(json!({
                "format": "PE",
                "is_64": pe.is_64,
                "entry": pe.entry,
                "libraries": pe.libraries,
                "exported_symbols": exports,
            }))
        }
        Ok(Object::Mach(mach)) => Ok(json!({
            "format": "Mach-O",
            "description": format!("{:?}", mach),
        })),
        Ok(Object::Archive(archive)) => Ok(json!({
            "format": "Archive",
            "members": archive.members(),
        })),
        _ => Ok(json!({
            "format": "raw_or_unknown",
            "size_bytes": bytes.len(),
        })),
    }
}

fn is_deep_inspection_tool(tool_name: &str) -> bool {
    let lower = tool_name.to_lowercase();
    lower.contains("decompile")
        || lower.contains("disassemble")
        || lower.contains("ast_dump")
        || lower.contains("pseudocode")
}

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
        // Enforce REA Law 2: Smallest Useful Inventory First
        if is_deep_inspection_tool(tool_name) {
            verify_inventory_populated_at(base_dir, slug)?;
        }

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
        self.call_tool_at(Path::new("."), slug, tool_name, arguments)
            .await
    }
}
