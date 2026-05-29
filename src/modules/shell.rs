use std::fs;
use std::process::{Command, Stdio};

pub fn install_zsh() -> anyhow::Result<()> {
    crate::distro::install_packages(&["zsh"])
}

pub fn install_oh_my_zsh() -> anyhow::Result<()> {
    let home = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("无法获取 home 目录"))?;
    let omz_dir = home.join(".oh-my-zsh");

    if omz_dir.exists() {
        return Ok(());
    }

    // Download install script to a temp file, then execute
    let tmp_script = "/tmp/linux-init-omz-install.sh";
    let download = Command::new("curl")
        .args(["-fsSL", "-o", tmp_script,
            "https://raw.githubusercontent.com/ohmyzsh/ohmyzsh/master/tools/install.sh"])
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()?;

    if !download.success() {
        anyhow::bail!("Oh My Zsh 安装脚本下载失败，请检查网络连接");
    }

    let status = Command::new("sh")
        .args([tmp_script, "", "--unattended"])
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()?;

    // Clean up temp file
    let _ = std::fs::remove_file(tmp_script);

    if !status.success() {
        anyhow::bail!("Oh My Zsh 安装失败");
    }
    Ok(())
}

pub fn set_theme(theme: &str) -> anyhow::Result<()> {
    let home = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("无法获取 home 目录"))?;
    let zshrc_path = home.join(".zshrc");

    let content = fs::read_to_string(&zshrc_path)?;
    let mut new_content = String::new();
    let mut found = false;

    for line in content.lines() {
        if line.trim().starts_with("ZSH_THEME=") {
            new_content.push_str(&format!("ZSH_THEME=\"{}\"", theme));
            found = true;
        } else {
            new_content.push_str(line);
        }
        new_content.push('\n');
    }

    if !found {
        new_content.push_str(&format!("\nZSH_THEME=\"{}\"\n", theme));
    }

    fs::write(&zshrc_path, new_content)?;
    Ok(())
}

pub fn set_plugins(plugins: &[String]) -> anyhow::Result<()> {
    let home = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("无法获取 home 目录"))?;
    let zshrc_path = home.join(".zshrc");

    let content = fs::read_to_string(&zshrc_path)?;
    let mut new_content = String::new();
    let mut found = false;
    let plugin_line = format!("plugins=({})", plugins.join(" "));

    for line in content.lines() {
        if line.trim().starts_with("plugins=") {
            new_content.push_str(&plugin_line);
            found = true;
        } else {
            new_content.push_str(line);
        }
        new_content.push('\n');
    }

    if !found {
        new_content.push_str(&format!("\n{}\n", plugin_line));
    }

    fs::write(&zshrc_path, new_content)?;
    Ok(())
}

#[allow(dead_code)]
pub fn install_third_party_plugin(name: &str) -> anyhow::Result<()> {
    let home = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("无法获取 home 目录"))?;
    let custom_plugins = home.join(".oh-my-zsh/custom/plugins");

    let plugin_dir = custom_plugins.join(name);
    if plugin_dir.exists() {
        return Ok(());
    }

    let repo_url = match name {
        "zsh-autosuggestions" => {
            "https://github.com/zsh-users/zsh-autosuggestions.git"
        }
        "zsh-syntax-highlighting" => {
            "https://github.com/zsh-users/zsh-syntax-highlighting.git"
        }
        "zsh-completions" => {
            "https://github.com/zsh-users/zsh-completions.git"
        }
        "fzf" => "https://github.com/chitoku-k/fzf-zsh-completions.git",
        "you-should-use" => {
            "https://github.com/MichaelAqworka/zsh-you-should-use.git"
        }
        _ => return Ok(()),
    };

    let status = Command::new("git")
        .args(["clone", repo_url, plugin_dir.to_str().unwrap()])
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()?;

    if !status.success() {
        anyhow::bail!("插件 {} 下载失败", name);
    }
    Ok(())
}

pub fn set_default_shell() -> anyhow::Result<()> {
    let which_zsh = Command::new("which").arg("zsh").output()?;
    let zsh_path = String::from_utf8_lossy(&which_zsh.stdout).trim().to_string();

    if zsh_path.is_empty() {
        anyhow::bail!("未找到 zsh");
    }

    let status = Command::new("chsh")
        .args(["-s", &zsh_path])
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()?;

    if !status.success() {
        anyhow::bail!("chsh 命令失败");
    }
    Ok(())
}

#[allow(dead_code)]
pub fn is_third_party_plugin_installed(name: &str) -> bool {
    let home = match dirs::home_dir() {
        Some(h) => h,
        None => return false,
    };
    home.join(".oh-my-zsh/custom/plugins").join(name).exists()
}

#[allow(dead_code)]
pub fn get_bundled_theme_list() -> Vec<(String, String)> {
    let home = match dirs::home_dir() {
        Some(h) => h,
        None => return vec![],
    };
    let themes_dir = home.join(".oh-my-zsh/themes");
    if !themes_dir.exists() {
        return vec![];
    }
    let mut themes = vec![];
    if let Ok(entries) = fs::read_dir(&themes_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map(|e| e == "zsh-theme").unwrap_or(false) {
                if let Some(name) = path.file_stem() {
                    themes.push((name.to_string_lossy().to_string(), String::new()));
                }
            }
        }
    }
    themes.sort_by(|a, b| a.0.cmp(&b.0));
    themes
}
