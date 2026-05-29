use std::path::PathBuf;
use std::process::{Command, Stdio};

fn get_real_home() -> anyhow::Result<PathBuf> {
    if let Ok(sudo_user) = std::env::var("SUDO_USER") {
        let output = Command::new("getent")
            .args(["passwd", &sudo_user])
            .output()?;
        let line = String::from_utf8_lossy(&output.stdout);
        if let Some(home) = line.split(':').nth(5) {
            return Ok(PathBuf::from(home.trim()));
        }
    }
    dirs::home_dir().ok_or_else(|| anyhow::anyhow!("无法获取 home 目录"))
}

pub fn is_nvm_installed() -> bool {
    nvm_dir().join("nvm.sh").exists()
}

fn nvm_dir() -> PathBuf {
    get_real_home()
        .unwrap_or_else(|_| dirs::home_dir().unwrap_or_default())
        .join(".nvm")
}

pub fn install_nvm() -> anyhow::Result<()> {
    // 确保依赖命令存在
    crate::utils::ensure_command("curl")?;
    
    let nvm_dir = nvm_dir();
    if nvm_dir.join("nvm.sh").exists() {
        return Ok(());
    }

    // Use the official install script
    let status = Command::new("bash")
        .arg("-c")
        .arg("curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.40.1/install.sh | bash")
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()?;

    if !status.success() {
        anyhow::bail!("nvm 安装脚本执行失败");
    }

    Ok(())
}

/// Load nvm in a subshell and install node LTS
pub fn install_node_lts() -> anyhow::Result<()> {
    let nvm_sh = nvm_dir().join("nvm.sh");
    if !nvm_sh.exists() {
        anyhow::bail!("nvm 未安装");
    }

    let script = format!(
        "source '{}' && nvm install --lts && nvm alias default lts/*",
        nvm_sh.display()
    );

    let status = Command::new("bash")
        .arg("-c")
        .arg(&script)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()?;

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

pub fn ensure_shell_integration() -> anyhow::Result<()> {
    // nvm install script usually adds this to .bashrc/.zshrc,
    // but let's make sure it's there for the active shell
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

    // Check and update both .bashrc and .zshrc if they exist
    for rc in [".bashrc", ".zshrc"] {
        let path = home.join(rc);
        if !path.exists() {
            continue;
        }
        let content = std::fs::read_to_string(&path).unwrap_or_default();
        if content.contains("NVM_DIR") {
            continue;
        }
        let mut f = std::fs::OpenOptions::new().append(true).open(&path)?;
        use std::io::Write;
        f.write_all(snippet.as_bytes())?;
    }

    Ok(())
}
