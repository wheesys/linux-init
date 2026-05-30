use crate::distro;
use crate::utils::get_real_home;
use std::fs;
use std::io::Write;
use std::process::Command;

/// 安装工具列表：逐个安装，包管理器优先，失败时走 GitHub 二进制下载兜底
/// 单个工具失败不影响后续安装，所有错误汇总返回
pub fn install_tools(selected: &[&str]) -> anyhow::Result<()> {
    let mut errors: Vec<String> = Vec::new();

    for tool in selected {
        if let Err(e) = install_single_tool(tool) {
            errors.push(format!("{}: {}", tool, e));
        }
    }

    if !errors.is_empty() {
        anyhow::bail!("以下工具安装失败:\n{}", errors.join("\n"));
    }
    Ok(())
}

/// 安装单个工具：包管理器 → GitHub 兜底
fn install_single_tool(tool: &str) -> anyhow::Result<()> {
    // 1. 尝试包管理器安装
    if let Some(pkg) = distro::package_name(tool) {
        if distro::is_package_installed(pkg) {
            return Ok(()); // 已安装
        }
        // 尝试安装，成功则返回
        if distro::install_packages(&[pkg]).is_ok() {
            return Ok(());
        }
    }

    // 2. 兜底：GitHub release 二进制下载（仅 Debian 系）
    if distro::detect().family() == crate::distro::DistroFamily::Debian {
        install_via_github_release(tool)?;
    } else {
        anyhow::bail!("{} 安装失败（包管理器和兜底方案均不可用）", tool);
    }
    Ok(())
}

/// GitHub release 二进制下载（不编译）
/// duf 提供 .deb，procs/eza 提供 tar.gz/zip
fn install_via_github_release(tool: &str) -> anyhow::Result<()> {
    match tool {
        "duf"   => install_github_deb("muesli", "duf", "duf")?,
        "procs" => install_github_tarball("dalance", "procs", "procs")?,
        "eza"   => install_github_tarball("eza-community", "eza", "eza")?,
        _ => anyhow::bail!("{} 在当前系统无可用安装方式", tool),
    }
    Ok(())
}

/// 从 GitHub release 下载 .deb 并 dpkg -i 安装
fn install_github_deb(owner: &str, repo: &str, bin_name: &str) -> anyhow::Result<()> {
    let tag = fetch_latest_tag(owner, repo)?;
    let deb_name = format!("{}_{}_linux_amd64.deb", bin_name, &tag[1..]); // tag 如 "v0.9.1", 去 v
    let url = format!(
        "https://github.com/{}/{}/releases/download/{}/{}",
        owner, repo, tag, deb_name
    );
    let tmp_deb = format!("/tmp/linux-init-{}.deb", bin_name);

    let status = Command::new("curl")
        .args(["-fsSL", "--max-time", "120", "-o", &tmp_deb, &url])
        .stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit())
        .status()?;
    if !status.success() {
        let _ = fs::remove_file(&tmp_deb);
        anyhow::bail!("{} 下载失败: {}", bin_name, url);
    }

    let status = Command::new("sudo")
        .args(["dpkg", "-i", &tmp_deb])
        .stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit())
        .status()?;
    let _ = fs::remove_file(&tmp_deb);
    if !status.success() {
        anyhow::bail!("{} deb 安装失败", bin_name);
    }
    Ok(())
}

/// 从 GitHub release 下载 tar.gz/zip 二进制，解压到 /usr/local/bin
fn install_github_tarball(owner: &str, repo: &str, bin_name: &str) -> anyhow::Result<()> {
    let tag = fetch_latest_tag(owner, repo)?;
    let arch = if cfg!(target_arch = "x86_64") { "x86_64" }
        else if cfg!(target_arch = "aarch64") { "aarch64" }
        else { anyhow::bail!("不支持的 CPU 架构") };

    let (tarball, _inner_path) = match (owner, repo) {
        ("dalance", "procs") => (
            format!("procs-v{}-{}-linux.zip", &tag[1..], arch),
            "procs".to_string(),
        ),
        ("eza-community", "eza") => (
            format!("eza_{}-unknown-linux-gnu.tar.gz", arch),
            "eza".to_string(),
        ),
        _ => anyhow::bail!("未知的 GitHub 仓库: {}/{}", owner, repo),
    };

    let url = format!(
        "https://github.com/{}/{}/releases/download/{}/{}",
        owner, repo, tag, tarball
    );
    let tmp_archive = format!("/tmp/linux-init-{}.archive", bin_name);
    let tmp_dir = format!("/tmp/linux-init-{}-extract", bin_name);

    // 下载
    let status = Command::new("curl")
        .args(["-fsSL", "--max-time", "120", "-o", &tmp_archive, &url])
        .stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit())
        .status()?;
    if !status.success() {
        let _ = fs::remove_file(&tmp_archive);
        anyhow::bail!("{} 下载失败: {}", bin_name, url);
    }

    // 解压
    let _ = fs::remove_dir_all(&tmp_dir);
    fs::create_dir_all(&tmp_dir)?;

    if tarball.ends_with(".zip") {
        let status = Command::new("unzip")
            .args(["-o", &tmp_archive, "-d", &tmp_dir])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::inherit())
            .status()?;
        if !status.success() {
            anyhow::bail!("{} 解压失败（需要 unzip 命令）", bin_name);
        }
    } else {
        let status = Command::new("tar")
            .args(["-xzf", &tmp_archive, "-C", &tmp_dir])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::inherit())
            .status()?;
        if !status.success() {
            anyhow::bail!("{} 解压失败", bin_name);
        }
    }

    // 找到二进制文件并安装到 /usr/local/bin
    let bin_path = find_binary(&tmp_dir, bin_name)?;
    let target = format!("/usr/local/bin/{}", bin_name);
    let status = Command::new("sudo")
        .args(["install", "-m", "755", &bin_path, &target])
        .stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit())
        .status()?;
    if !status.success() {
        anyhow::bail!("{} 安装到 /usr/local/bin 失败", bin_name);
    }

    // 清理
    let _ = fs::remove_file(&tmp_archive);
    let _ = fs::remove_dir_all(&tmp_dir);
    Ok(())
}

/// 从解压目录递归查找二进制文件
fn find_binary(dir: &str, name: &str) -> anyhow::Result<String> {
    let entries = fs::read_dir(dir)?;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if let Ok(found) = find_binary(path.to_str().unwrap_or(""), name) {
                return Ok(found);
            }
        } else if let Some(fname) = path.file_name() {
            if fname == name {
                return Ok(path.to_str().unwrap_or("").to_string());
            }
        }
    }
    anyhow::bail!("未在解压目录中找到二进制: {}", name)
}

/// 通过 GitHub API 获取最新 release tag
fn fetch_latest_tag(owner: &str, repo: &str) -> anyhow::Result<String> {
    let api_url = format!(
        "https://api.github.com/repos/{}/{}/releases/latest",
        owner, repo
    );
    let output = Command::new("curl")
        .args(["-fsSL", "--max-time", "30", &api_url])
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::inherit())
        .output()?;
    if !output.status.success() {
        anyhow::bail!("获取 {}/{} 最新版本信息失败", owner, repo);
    }
    let json = String::from_utf8_lossy(&output.stdout);
    // 解析 "tag_name": "v0.8.1"
    let tag = json
        .lines()
        .find_map(|line| {
            let trimmed = line.trim();
            if trimmed.starts_with("\"tag_name\":") {
                Some(
                    trimmed
                        .trim_start_matches("\"tag_name\":")
                        .trim()
                        .trim_matches('"')
                        .trim_matches(',')
                        .to_string(),
                )
            } else {
                None
            }
        })
        .ok_or_else(|| anyhow::anyhow!("无法解析 {} 的 tag_name", api_url))?;
    Ok(tag)
}

pub fn get_tool_status(tool: &str) -> bool {
    if let Some(pkg) = distro::package_name(tool) {
        distro::is_package_installed(pkg)
    } else {
        false
    }
}

/// 确定 shell 配置目标：优先使用系统当前默认 shell（$SHELL），
/// 若无法识别则 fallback 到用户本次会话选定的 shell。
/// 返回 "bash" 或 "zsh"，两者都不确定时返回 None。
pub fn resolve_shell_for_config(selected_shell: Option<&str>) -> Option<String> {
    let current = std::env::var("SHELL").unwrap_or_default();
    if current.contains("zsh") {
        return Some("zsh".to_string());
    }
    if current.contains("bash") {
        return Some("bash".to_string());
    }
    selected_shell.map(|s| s.to_string())
}

/// 配置命令别名，只写入目标 shell 的配置文件
pub fn configure_aliases(installed_tools: &[&str], selected_shell: Option<&str>) -> anyhow::Result<()> {
    let home = get_real_home()?;

    let mut aliases = Vec::new();
    if installed_tools.contains(&"trash-cli") {
        aliases.push(r#"alias rm='trash '"#);
    }
    if installed_tools.contains(&"procs") {
        aliases.push(r#"alias ps='procs '"#);
    }
    if installed_tools.contains(&"dust") {
        aliases.push(r#"alias du='dust '"#);
    }
    if installed_tools.contains(&"duf") {
        aliases.push(r#"alias df='duf '"#);
    }
    if installed_tools.contains(&"eza") {
        aliases.push(r#"alias la='eza -lah'"#);
    }

    if aliases.is_empty() {
        return Ok(());
    }

    let target = resolve_shell_for_config(selected_shell);
    match target.as_deref() {
        Some("zsh") => {
            let path = home.join(".zshrc");
            if path.exists() {
                append_aliases_to_file(&path, &aliases, "zsh")?;
            }
        }
        _ => {
            let path = home.join(".bashrc");
            if path.exists() {
                append_aliases_to_file(&path, &aliases, "bash")?;
            }
        }
    }

    Ok(())
}

fn append_aliases_to_file(path: &std::path::Path, aliases: &[&str], shell: &str) -> anyhow::Result<()> {
    let content = fs::read_to_string(path)?;

    let marker = format!("# Linux Init - {} aliases", shell);
    if content.contains(&marker) {
        return Ok(());
    }

    let mut file = fs::OpenOptions::new()
        .append(true)
        .open(path)?;

    writeln!(file, "\n{}", marker)?;
    for alias in aliases {
        writeln!(file, "{}", alias)?;
    }

    Ok(())
}

/// 配置 direnv hook，只写入目标 shell 的配置文件
pub fn configure_direnv_hook(selected_shell: Option<&str>) -> anyhow::Result<()> {
    let home = get_real_home()?;

    if !distro::is_package_installed(distro::package_name("direnv").unwrap_or("direnv")) {
        return Ok(());
    }

    let target = resolve_shell_for_config(selected_shell);
    match target.as_deref() {
        Some("zsh") => {
            let path = home.join(".zshrc");
            if path.exists() {
                append_direnv_hook_to_file(&path, "zsh")?;
            }
        }
        _ => {
            let path = home.join(".bashrc");
            if path.exists() {
                append_direnv_hook_to_file(&path, "bash")?;
            }
        }
    }

    Ok(())
}

fn append_direnv_hook_to_file(path: &std::path::Path, shell: &str) -> anyhow::Result<()> {
    let content = fs::read_to_string(path)?;

    let marker = format!("# Linux Init - direnv hook for {}", shell);
    if content.contains(&marker) {
        return Ok(());
    }

    let mut file = fs::OpenOptions::new()
        .append(true)
        .open(path)?;

    writeln!(file, "\n{}", marker)?;
    writeln!(file, "eval \"$(direnv hook {})\"", shell)?;

    Ok(())
}

pub fn clear_tools(selected: &[bool]) -> anyhow::Result<()> {
    let pkg_list: Vec<&str> = selected.iter().enumerate()
        .filter(|(_, &s)| s)
        .filter_map(|(i, _)| crate::distro::package_name(crate::app::TOOLS[i].0))
        .collect();
    if !pkg_list.is_empty() {
        crate::distro::uninstall_packages(&pkg_list)?;
    }
    Ok(())
}
