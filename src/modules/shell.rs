use std::fs;
use std::process::{Command, Stdio};

pub fn install_zsh() -> anyhow::Result<()> {
    crate::distro::install_packages(&["zsh"])
}

pub fn install_oh_my_zsh() -> anyhow::Result<()> {
    let home = get_real_home()?;
    let omz_dir = home.join(".oh-my-zsh");
    
    // 创建日志目录
    let log_dir = home.join(".config").join("linux-init");
    std::fs::create_dir_all(&log_dir)?;
    let log_path = log_dir.join("omz-install.log");
    
    let mut log = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)?;

    use std::io::Write;
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    writeln!(log, "\n=== OMZ Install Start: {} ===", timestamp)?;
    writeln!(log, "Real home: {:?}", home)?;
    writeln!(log, "OMZ dir: {:?}", omz_dir)?;

    if omz_dir.exists() {
        writeln!(log, "OMZ already exists, skipping")?;
        return Ok(());
    }

    // 下载脚本 - 使用 inherit 让用户看到进度，加超时
    let tmp_script = "/tmp/linux-init-omz-install.sh";
    writeln!(log, "Downloading install script to {}...", tmp_script)?;
    log.flush()?;
    
    // 先检测网络连通性
    let ping = Command::new("timeout")
        .args(["5", "curl", "-sSL", "--head", "https://raw.githubusercontent.com"])
        .output();
    
    if ping.is_err() || !ping.as_ref().unwrap().status.success() {
        writeln!(log, "Network check failed")?;
        anyhow::bail!("无法连接到 github.com，请检查网络连接");
    }
    
    let download = Command::new("timeout")
        .args(["60", "curl", "-fsSL", "-o", tmp_script,
            "https://raw.githubusercontent.com/ohmyzsh/ohmyzsh/master/tools/install.sh"])
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()?;

    writeln!(log, "Download exit code: {:?}", download.code())?;
    if !download.success() {
        let code = download.code().unwrap_or(-1);
        writeln!(log, "Download failed with code {}", code)?;
        if code == 124 {
            anyhow::bail!("下载超时（60秒），请检查网络连接");
        }
        anyhow::bail!("Oh My Zsh 安装脚本下载失败 (exit code: {})", code);
    }

    writeln!(log, "Download successful, script size: {} bytes", 
        std::fs::metadata(tmp_script).map(|m| m.len()).unwrap_or(0))?;
    log.flush()?;

    // 执行安装 - 使用 inherit 让用户看到进度
    writeln!(log, "Checking SUDO_USER: {:?}", std::env::var("SUDO_USER"))?;
    log.flush()?;
    
    let status = if let Ok(sudo_user) = std::env::var("SUDO_USER") {
        writeln!(log, "Running as user: {}", sudo_user)?;
        let cmd = format!("sudo -u {} sh {} \"\" --unattended", sudo_user, tmp_script);
        writeln!(log, "Command: {}", cmd)?;
        log.flush()?;
        
        let status = Command::new("sudo")
            .args(["-u", &sudo_user, "sh", tmp_script, "", "--unattended"])
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()?;
        
        writeln!(log, "Install exit code: {:?}", status.code())?;
        status
    } else {
        writeln!(log, "Running as current user (no SUDO_USER)")?;
        log.flush()?;
        
        let status = Command::new("sh")
            .args([tmp_script, "", "--unattended"])
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()?;
        
        writeln!(log, "Install exit code: {:?}", status.code())?;
        status
    };
    log.flush()?;

    // 清理
    let _ = std::fs::remove_file(tmp_script);
    writeln!(log, "Cleaned up temp script")?;

    // 修复权限
    if let Ok(sudo_user) = std::env::var("SUDO_USER") {
        writeln!(log, "Fixing ownership for {}...", sudo_user)?;
        let chown_status = Command::new("chown")
            .args(["-R", &format!("{}:{}", sudo_user, sudo_user), 
                omz_dir.to_str().unwrap_or("")])
            .status()?;
        writeln!(log, "Chown exit code: {:?}", chown_status.code())?;
    }

    // 检查安装结果
    let installed = omz_dir.exists();
    writeln!(log, "OMZ dir exists after install: {}", installed)?;
    writeln!(log, "=== OMZ Install End ===\n")?;

    if !status.success() || !installed {
        anyhow::bail!("Oh My Zsh 安装失败。查看日志: {:?}", log_path);
    }
    
    Ok(())
}

fn get_real_home() -> anyhow::Result<std::path::PathBuf> {
    if let Ok(sudo_user) = std::env::var("SUDO_USER") {
        let output = Command::new("getent")
            .args(["passwd", &sudo_user])
            .output()?;
        let line = String::from_utf8_lossy(&output.stdout);
        if let Some(home) = line.split(':').nth(5) {
            return Ok(std::path::PathBuf::from(home.trim()));
        }
    }
    dirs::home_dir().ok_or_else(|| anyhow::anyhow!("无法获取 home 目录"))
}

pub fn set_theme(theme: &str) -> anyhow::Result<()> {
    let home = get_real_home()?;
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
    let home = get_real_home()?;
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
    let home = get_real_home()?;
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

    // Fix ownership if running with sudo
    if let Ok(sudo_user) = std::env::var("SUDO_USER") {
        let _ = Command::new("chown")
            .args(["-R", &format!("{}:{}", sudo_user, sudo_user), plugin_dir.to_str().unwrap_or("")])
            .status();
    }

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
    let home = match get_real_home() {
        Ok(h) => h,
        Err(_) => return false,
    };
    home.join(".oh-my-zsh/custom/plugins").join(name).exists()
}

#[allow(dead_code)]
pub fn get_bundled_theme_list() -> Vec<(String, String)> {
    let home = match get_real_home() {
        Ok(h) => h,
        Err(_) => return vec![],
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
