use std::process::Command;

/// 刷新 dnf 缓存
pub fn update() -> anyhow::Result<()> {
    let status = Command::new("sudo")
        .args(["dnf", "makecache"])
        .status()?;
    if !status.success() {
        anyhow::bail!("dnf makecache 失败");
    }
    Ok(())
}

/// 安装包
pub fn install(packages: &[&str]) -> anyhow::Result<()> {
    if packages.is_empty() {
        return Ok(());
    }
    let status = Command::new("sudo")
        .arg("dnf")
        .arg("install")
        .arg("-y")
        .args(packages)
        .status()?;
    if !status.success() {
        anyhow::bail!("dnf 安装失败");
    }
    Ok(())
}

/// 卸载包
pub fn uninstall(packages: &[&str]) -> anyhow::Result<()> {
    if packages.is_empty() {
        return Ok(());
    }
    let status = Command::new("sudo")
        .args(["dnf", "remove", "-y"])
        .args(packages)
        .status()?;
    if !status.success() {
        anyhow::bail!("dnf 卸载失败");
    }
    Ok(())
}

/// 检查包是否已安装（rpm -q 返回 0 表示已安装）
pub fn is_installed(package: &str) -> bool {
    Command::new("rpm")
        .arg("-q")
        .arg(package)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// 检查包是否在 dnf 仓库中可用
pub fn package_exists(package: &str) -> bool {
    Command::new("dnf")
        .args(["list", "available", package])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// 工具名 → Fedora 包名映射
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
        // docker 使用官方脚本安装，不走 dnf
        "docker" => None,
        "docker-compose" => None,
        "noto-fonts-cjk" => Some("google-noto-sans-cjk-fonts"),
        "wqy-microhei" => Some("wqy-microhei-fonts"),
        "wqy-zenhei" => Some("wqy-zenhei-fonts"),
        "fcitx5" => Some("fcitx5"),
        "fcitx5-chinese-addons" => Some("fcitx5-chinese-addons"),
        "fcitx5-configtool" => Some("fcitx5-configtool"),
        "vim" => Some("vim-enhanced"),
        "openssh-server" => Some("openssh-server"),
        "glibc-langpack-zh" => Some("glibc-langpack-zh"),
        "fzf" => Some("fzf"),
        _ => None,
    }
}
