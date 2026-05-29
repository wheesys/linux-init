use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    pub language: Option<String>,
    pub completed: CompletedModules,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CompletedModules {
    pub zsh_installed: bool,
    pub omz_installed: bool,
    pub zsh_theme: Option<String>,
    pub zsh_plugins: Vec<String>,
    pub zsh_default: bool,

    pub docker_installed: bool,
    pub docker_compose_installed: bool,
    pub docker_user_configured: bool,

    pub ssh_key_generated: bool,
    pub ssh_key_type: Option<String>,

    pub tools_installed: Vec<String>,

    pub vim_installed: bool,
    pub vundle_installed: bool,
    pub vim_plugins: Vec<String>,

    pub nvm_installed: bool,
    pub node_installed: bool,

    pub ssh_server_installed: bool,
    pub ssh_server_configured: bool,

    pub chinese_locale_configured: bool,
    pub chinese_fonts_installed: bool,
    pub fcitx5_installed: bool,
}

impl Config {
    pub fn load() -> Self {
        if let Some(path) = Self::config_path() {
            if path.exists() {
                if let Ok(content) = fs::read_to_string(&path) {
                    if let Ok(config) = serde_json::from_str(&content) {
                        return config;
                    }
                }
            }
        }
        Self::default()
    }

    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(path) = Self::config_path() {
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)?;
                
                // If running with sudo, fix ownership to the real user
                if let Ok(sudo_user) = std::env::var("SUDO_USER") {
                    let _ = std::process::Command::new("chown")
                        .args([&format!("{}:{}", sudo_user, sudo_user), parent.to_str().unwrap_or("")])
                        .status();
                }
            }
            let content = serde_json::to_string_pretty(self)?;
            fs::write(&path, content)?;
            
            // If running with sudo, fix file ownership
            if let Ok(sudo_user) = std::env::var("SUDO_USER") {
                let _ = std::process::Command::new("chown")
                    .args([&format!("{}:{}", sudo_user, sudo_user), path.to_str().unwrap_or("")])
                    .status();
            }
        }
        Ok(())
    }

    /// Get config path, using SUDO_USER's home if running with sudo
    fn config_path() -> Option<PathBuf> {
        let home = if let Ok(sudo_user) = std::env::var("SUDO_USER") {
            // Running with sudo, get the real user's home
            dirs::home_dir().and_then(|_| {
                // Try to get SUDO_USER's home from /etc/passwd
                let output = std::process::Command::new("getent")
                    .args(["passwd", &sudo_user])
                    .output()
                    .ok()?;
                let line = String::from_utf8_lossy(&output.stdout);
                let home = line.split(':').nth(5)?;
                Some(PathBuf::from(home))
            })
        } else {
            dirs::home_dir()
        };

        home.map(|h| h.join(".config").join("linux-init").join("config.json"))
    }
}
