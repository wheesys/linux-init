use std::process::Command;

/// 检查命令是否存在
pub fn command_exists(cmd: &str) -> bool {
    Command::new("which")
        .arg(cmd)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// 确保命令存在，不存在则安装
pub fn ensure_command(cmd: &str) -> anyhow::Result<()> {
    if command_exists(cmd) {
        return Ok(());
    }

    // 命令名到包名的映射
    let package = match cmd {
        "curl" => "curl",
        "git" => "git",
        "vim" => "vim",
        _ => cmd,
    };

    crate::distro::install_packages(&[package])
}
