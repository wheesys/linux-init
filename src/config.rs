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
    pub docker_service_running: bool,

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

                if let Ok(sudo_user) = std::env::var("SUDO_USER") {
                    let _ = std::process::Command::new("chown")
                        .args([&format!("{}:{}", sudo_user, sudo_user), parent.to_str().unwrap_or("")])
                        .status();
                }
            }
            let content = serde_json::to_string_pretty(self)?;
            fs::write(&path, content)?;

            if let Ok(sudo_user) = std::env::var("SUDO_USER") {
                let _ = std::process::Command::new("chown")
                    .args([&format!("{}:{}", sudo_user, sudo_user), path.to_str().unwrap_or("")])
                    .status();
            }
        }
        Ok(())
    }

    fn config_path() -> Option<PathBuf> {
        let home = crate::utils::get_real_home().ok()?;
        Some(home.join(".config").join("linux-init").join("config.json"))
    }
}
