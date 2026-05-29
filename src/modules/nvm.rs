use std::path::PathBuf;
use std::process::Command;
use crate::utils::{get_real_home, DownloadLogger};

pub fn is_nvm_installed() -> bool {
    nvm_dir().join("nvm.sh").exists()
}

fn nvm_dir() -> PathBuf {
    get_real_home()
        .unwrap_or_else(|_| dirs::home_dir().unwrap_or_default())
        .join(".nvm")
}

pub fn install_nvm() -> anyhow::Result<()> {
    crate::utils::ensure_command("curl")?;

    let _home = get_real_home()?;
    let nvm_path = nvm_dir();

    let mut log = DownloadLogger::new("nvm-install.log")?;
    log.log(&format!("NVM dir: {:?}", nvm_path))?;

    if nvm_path.join("nvm.sh").exists() {
        log.log("NVM already exists, skipping")?;
        log.finish(true);
        return Ok(());
    }

    let sources = [
        ("GitHub", "https://raw.githubusercontent.com/nvm-sh/nvm/v0.40.1/install.sh"),
        ("Gitee", "https://gitee.com/mirrors/nvm/raw/v0.40.1/install.sh"),
    ];

    let tmp_script = "/tmp/linux-init-nvm-install.sh";
    let mut downloaded = false;

    for (name, url) in &sources {
        log.log(&format!("Trying {} mirror: {}", name, url))?;
        if log.check_network(url).is_err() {
            log.log(&format!("{} check failed, trying next...", name))?;
            continue;
        }

        let status = log.run_download(
            &format!("Download from {}", name),
            "curl",
            &["-fSL", "--max-time", "60", "-o", tmp_script, url],
        )?;

        if status.success() {
            downloaded = true;
            break;
        }
        log.log(&format!("{} download failed, trying next...", name))?;
    }

    if !downloaded {
        anyhow::bail!("NVM 安装脚本下载失败，所有镜像均失败");
    }

    // 执行安装
    let install_status = if let Ok(sudo_user) = std::env::var("SUDO_USER") {
        log.run_as_user("Install NVM", &sudo_user, "bash", &[tmp_script])?
    } else {
        log.run_download("Install NVM", "bash", &[tmp_script])?
    };

    let _ = std::fs::remove_file(tmp_script);
    log.log("Cleaned up temp script")?;

    log.fix_ownership(nvm_path.to_str().unwrap_or(""));

    let installed = nvm_path.join("nvm.sh").exists();
    log.finish(installed && install_status.success());

    if !install_status.success() || !installed {
        anyhow::bail!("NVM 安装失败");
    }

    // 安装成功后自动配置 shell 集成（安装时无会话选定 shell，依赖 $SHELL）
    log.log("Auto-configuring shell integration...")?;
    ensure_shell_integration(None)?;

    Ok(())
}

pub fn install_node_latest() -> anyhow::Result<()> {
    let nvm_sh = nvm_dir().join("nvm.sh");
    if !nvm_sh.exists() {
        anyhow::bail!("nvm 未安装");
    }

    let mut log = DownloadLogger::new("node-latest-install.log")?;

    let script = format!(
        "source '{}' && nvm install node && nvm alias default node",
        nvm_sh.display()
    );

    log.log("Installing Node.js latest")?;

    let status = if let Ok(sudo_user) = std::env::var("SUDO_USER") {
        log.run_as_user("nvm install node", &sudo_user, "bash", &["-c", &script])?
    } else {
        log.run_download("nvm install node", "bash", &["-c", &script])?
    };

    log.finish(status.success());

    if !status.success() {
        anyhow::bail!("Node.js 最新版安装失败");
    }
    Ok(())
}

pub fn install_node_lts() -> anyhow::Result<()> {
    let nvm_sh = nvm_dir().join("nvm.sh");
    if !nvm_sh.exists() {
        anyhow::bail!("nvm 未安装");
    }

    let mut log = DownloadLogger::new("node-lts-install.log")?;

    let script = format!(
        "source '{}' && nvm install --lts && nvm alias default lts/*",
        nvm_sh.display()
    );

    log.log("Installing Node.js LTS")?;

    let status = if let Ok(sudo_user) = std::env::var("SUDO_USER") {
        log.run_as_user("nvm install --lts", &sudo_user, "bash", &["-c", &script])?
    } else {
        log.run_download("nvm install --lts", "bash", &["-c", &script])?
    };

    log.finish(status.success());

    if !status.success() {
        anyhow::bail!("Node.js LTS 安装失败");
    }
    Ok(())
}

pub fn installed_node_version() -> Option<String> {
    let nvm_sh = nvm_dir().join("nvm.sh");
    if !nvm_sh.exists() {
        return None;
    }

    let script = format!("source '{}' && node --version 2>/dev/null", nvm_sh.display());
    let output = Command::new("bash").arg("-c").arg(&script).output().ok()?;

    if output.status.success() {
        let ver = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if ver.is_empty() {
            None
        } else {
            Some(ver)
        }
    } else {
        None
    }
}

pub fn ensure_shell_integration(selected_shell: Option<&str>) -> anyhow::Result<()> {
    let home = get_real_home()?;
    let nvm_sh = nvm_dir().join("nvm.sh");

    let snippet = format!(
        r#"
export NVM_DIR="$HOME/.nvm"
[ -s "{}" ] && \. "{}"
[ -s "$NVM_DIR/bash_completion" ] && \. "$NVM_DIR/bash_completion"
"#,
        nvm_sh.display(),
        nvm_sh.display()
    );

    let target = crate::modules::tools::resolve_shell_for_config(selected_shell);
    let rc_file = match target.as_deref() {
        Some("zsh") => ".zshrc",
        _ => ".bashrc",
    };

    let path = home.join(rc_file);
    if !path.exists() {
        return Ok(());
    }
    let content = std::fs::read_to_string(&path).unwrap_or_default();
    if content.contains("NVM_DIR") {
        return Ok(());
    }
    let mut f = std::fs::OpenOptions::new().append(true).open(&path)?;
    use std::io::Write;
    f.write_all(snippet.as_bytes())?;

    Ok(())
}

pub fn clear_nvm() -> anyhow::Result<()> {
    let home = crate::utils::get_real_home()?;
    let nvm_dir = home.join(".nvm");
    if nvm_dir.exists() {
        std::fs::remove_dir_all(&nvm_dir)?;
    }
    for rc in &[".bashrc", ".zshrc"] {
        let path = home.join(rc);
        if path.exists() {
            if let Ok(content) = std::fs::read_to_string(&path) {
                let filtered: Vec<&str> = content.lines()
                    .filter(|l| !l.contains("NVM_DIR") && !l.contains("nvm.sh"))
                    .collect();
                let _ = std::fs::write(&path, filtered.join("\n") + "\n");
            }
        }
    }
    Ok(())
}
