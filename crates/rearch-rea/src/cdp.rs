use anyhow::Result;
use std::path::{Path, PathBuf};
use std::process::{Child, Command};

#[derive(Debug, Clone)]
pub struct CdpConfig {
    pub host: String,
    pub port: u16,
    pub headless: bool,
}

impl Default for CdpConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 9222,
            headless: true,
        }
    }
}

impl CdpConfig {
    pub fn endpoint_url(&self) -> String {
        format!("http://{}:{}/json/version", self.host, self.port)
    }

    pub fn to_command_args(&self, user_data_dir: PathBuf) -> Vec<String> {
        let mut args = vec![
            format!("--remote-debugging-port={}", self.port),
            format!("--remote-debugging-address={}", self.host),
            format!("--user-data-dir={}", user_data_dir.display()),
            "--no-first-run".to_string(),
            "--no-default-browser-check".to_string(),
        ];
        if self.headless {
            args.push("--headless=new".to_string());
        }
        args
    }
}

pub struct CdpManager {
    config: CdpConfig,
    child: Option<Child>,
    user_data_dir: PathBuf,
}

impl CdpManager {
    pub fn new(config: CdpConfig, user_data_dir: PathBuf) -> Self {
        Self {
            config,
            child: None,
            user_data_dir,
        }
    }

    pub fn config(&self) -> &CdpConfig {
        &self.config
    }

    pub fn is_alive(&self) -> bool {
        // Simple TCP connection or HTTP probe to endpoint
        if let Ok(resp) =
            std::net::TcpStream::connect(format!("{}:{}", self.config.host, self.config.port))
        {
            drop(resp);
            true
        } else {
            false
        }
    }

    pub fn spawn_browser(&mut self, browser_bin: &Path) -> Result<()> {
        let args = self.config.to_command_args(self.user_data_dir.clone());
        let child = Command::new(browser_bin).args(&args).spawn()?;
        self.child = Some(child);
        Ok(())
    }

    pub fn terminate(&mut self) -> Result<()> {
        if let Some(mut child) = self.child.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
        Ok(())
    }
}

impl Drop for CdpManager {
    fn drop(&mut self) {
        let _ = self.terminate();
    }
}
