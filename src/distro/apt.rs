use std::process::Command;

/// 刷新 apt 源（整个安装流程只应调用一次）
pub fn update() -> anyhow::Result<()> {
    let status = Command::new("sudo")
        .arg("apt")
        .arg("update")
        .status()?;
    if !status.success() {
        anyhow::bail!("apt update 失败");
    }
    Ok(())
}

/// 安装包（不自动 update，调用前应先 update）
pub fn install(packages: &[&str]) -> anyhow::Result<()> {
    if packages.is_empty() {
        return Ok(());
    }
    let status = Command::new("sudo")
        .arg("apt")
        .arg("install")
        .arg("-y")
        .args(packages)
        .status()?;
    if !status.success() {
        anyhow::bail!("apt 安装失败");
    }
    Ok(())
}

pub fn uninstall(packages: &[&str]) -> anyhow::Result<()> {
    if packages.is_empty() { return Ok(()); }
    let status = Command::new("sudo")
        .arg("apt").arg("remove").arg("-y").args(packages).status()?;
    if !status.success() { anyhow::bail!("apt 卸载失败"); }
    Ok(())
}

pub fn is_installed(package: &str) -> bool {
    Command::new("dpkg")
        .arg("-s")
        .arg(package)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// 检查包是否在 apt 仓库中存在（不安装）
pub fn package_exists(package: &str) -> bool {
    Command::new("apt-cache")
        .args(["show", package])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

pub fn package_name(tool: &str) -> Option<&'static str> {
    match tool {
        "git" => Some("git"),
        "curl" => Some("curl"),
        "wget" => Some("wget"),
        "btop" => Some("btop"),
        "neovim" => Some("neovim"),
        "tmux" => Some("tmux"),
        "jq" => Some("jq"),
        "ripgrep" => Some("ripgrep"),
        "fd" => Some("fd-find"),
        "bat" => Some("bat"),
        "eza" => Some("eza"),
        "trash-cli" => Some("trash-cli"),
        "procs" => Some("procs"),
        "dust" => Some("du-dust"),
        "duf" => Some("duf"),
        "direnv" => Some("direnv"),
        "zsh" => Some("zsh"),
        // docker 使用官方脚本安装，不走 apt
        "docker" => None,
        "docker-compose" => None,
        "noto-fonts-cjk" => Some("fonts-noto-cjk"),
        "wqy-microhei" => Some("fonts-wqy-microhei"),
        "wqy-zenhei" => Some("fonts-wqy-zenhei"),
        "fcitx5" => Some("fcitx5"),
        "fcitx5-chinese-addons" => Some("fcitx5-chinese-addons"),
        "fcitx5-configtool" => Some("fcitx5-configtool"),
        "vim" => Some("vim"),
        "openssh-server" => Some("openssh-server"),
        _ => None,
    }
}
