use std::net::{SocketAddr, ToSocketAddrs, TcpStream};
use std::process::Command;
use std::time::{Duration, Instant};
use std::fs;

use crate::distro::{self, DistroFamily, Distro};

// ── 镜像数据结构 ─────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct MirrorEntry {
    pub name_cn: &'static str,
    pub name_en: &'static str,
    /// 镜像 URL（空串表示恢复默认）
    pub url: String,
    /// TCP 测速用的 host:port（空串跳过测速）
    pub ping_host: &'static str,
}

// ── 系统源 (pacman) ──────────────────────────────────────

static PACMAN_PING_HOSTS: &[&str] = &[
    "",
    "mirrors.tuna.tsinghua.edu.cn:443",
    "mirrors.ustc.edu.cn:443",
    "mirrors.aliyun.com:443",
    "mirrors.cloud.tencent.com:443",
];

static PACMAN_SERVER_URLS: &[&str] = &[
    "",
    "https://mirrors.tuna.tsinghua.edu.cn/archlinux/$repo/os/$arch",
    "https://mirrors.ustc.edu.cn/archlinux/$repo/os/$arch",
    "https://mirrors.aliyun.com/archlinux/$repo/os/$arch",
    "https://mirrors.cloud.tencent.com/archlinux/$repo/os/$arch",
];

pub fn pacman_mirrors() -> Vec<MirrorEntry> {
    let names_cn = ["默认官方源", "清华 (TUNA)", "中科大 (USTC)", "阿里云", "腾讯云"];
    let names_en = ["Default Official", "Tsinghua (TUNA)", "USTC", "Aliyun", "Tencent Cloud"];
    names_cn.iter().enumerate().map(|(i, cn)| MirrorEntry {
        name_cn: cn,
        name_en: names_en[i],
        url: PACMAN_SERVER_URLS[i].to_string(),
        ping_host: PACMAN_PING_HOSTS[i],
    }).collect()
}

// ── 系统源 (apt) ────────────────────────────────────────

static APT_HOSTS: &[&str] = &[
    "",
    "mirrors.tuna.tsinghua.edu.cn",
    "mirrors.ustc.edu.cn",
    "mirrors.aliyun.com",
    "mirrors.cloud.tencent.com",
];

pub fn apt_mirrors() -> Vec<MirrorEntry> {
    let distro = distro::detect();
    let distro_path = match &distro {
        Distro::Ubuntu(_) => "ubuntu",
        Distro::Debian(_) => "debian",
        _ => "ubuntu",
    };
    let names_cn = ["默认官方源", "清华 (TUNA)", "中科大 (USTC)", "阿里云", "腾讯云"];
    let names_en = ["Default Official", "Tsinghua (TUNA)", "USTC", "Aliyun", "Tencent Cloud"];
    let ping_hosts = ["", "mirrors.tuna.tsinghua.edu.cn:443", "mirrors.ustc.edu.cn:443", "mirrors.aliyun.com:443", "mirrors.cloud.tencent.com:443"];
    names_cn.iter().enumerate().map(|(i, cn)| MirrorEntry {
        name_cn: cn,
        name_en: names_en[i],
        url: if APT_HOSTS[i].is_empty() { String::new() }
             else { format!("https://{}/{}", APT_HOSTS[i], distro_path) },
        ping_host: ping_hosts[i],
    }).collect()
}

// ── 系统源 (dnf) ────────────────────────────────────────

static DNF_HOSTS: &[&str] = &[
    "",
    "mirrors.tuna.tsinghua.edu.cn",
    "mirrors.ustc.edu.cn",
    "mirrors.aliyun.com",
    "mirrors.cloud.tencent.com",
];

pub fn dnf_mirrors() -> Vec<MirrorEntry> {
    let names_cn = ["默认官方源", "清华 (TUNA)", "中科大 (USTC)", "阿里云", "腾讯云"];
    let names_en = ["Default Official", "Tsinghua (TUNA)", "USTC", "Aliyun", "Tencent Cloud"];
    let ping_hosts = ["", "mirrors.tuna.tsinghua.edu.cn:443", "mirrors.ustc.edu.cn:443", "mirrors.aliyun.com:443", "mirrors.cloud.tencent.com:443"];
    names_cn.iter().enumerate().map(|(i, cn)| MirrorEntry {
        name_cn: cn,
        name_en: names_en[i],
        // URL 存储镜像主机名，切换时按主机名替换 repo 文件
        url: DNF_HOSTS[i].to_string(),
        ping_host: ping_hosts[i],
    }).collect()
}

// ── Docker 源 ────────────────────────────────────────────

pub fn docker_mirrors() -> Vec<MirrorEntry> {
    vec![
        MirrorEntry { name_cn: "默认（无镜像加速）", name_en: "Default (no mirror)", url: String::new(), ping_host: "" },
        MirrorEntry { name_cn: "阿里云（需替换 your-id）", name_en: "Aliyun (replace your-id)", url: "https://<your-id>.mirror.aliyuncs.com".into(), ping_host: "<your-id>.mirror.aliyuncs.com:443" },
        MirrorEntry { name_cn: "腾讯云", name_en: "Tencent Cloud", url: "https://mirror.ccs.tencentyun.com".into(), ping_host: "mirror.ccs.tencentyun.com:443" },
        MirrorEntry { name_cn: "Docker China", name_en: "Docker China", url: "https://registry.docker-cn.com".into(), ping_host: "registry.docker-cn.com:443" },
    ]
}

// ── Node (npm) 源 ────────────────────────────────────────

pub fn npm_mirrors() -> Vec<MirrorEntry> {
    vec![
        MirrorEntry { name_cn: "默认 (npmjs.org)", name_en: "Default (npmjs.org)", url: "https://registry.npmjs.org".into(), ping_host: "registry.npmjs.org:443" },
        MirrorEntry { name_cn: "npmmirror.com", name_en: "npmmirror.com", url: "https://registry.npmmirror.com".into(), ping_host: "registry.npmmirror.com:443" },
        MirrorEntry { name_cn: "腾讯云 npm", name_en: "Tencent Cloud npm", url: "https://mirrors.cloud.tencent.com/npm/".into(), ping_host: "mirrors.cloud.tencent.com:443" },
    ]
}

// ── 网络测速 ─────────────────────────────────────────────

fn ping_host(host: &str) -> Option<u64> {
    if host.is_empty() {
        return None;
    }
    let start = Instant::now();
    let addr: SocketAddr = match host.to_socket_addrs().ok()?.next() {
        Some(a) => a,
        None => return None,
    };
    match TcpStream::connect_timeout(&addr, Duration::from_secs(3)) {
        Ok(_) => Some(start.elapsed().as_millis() as u64),
        Err(_) => None,
    }
}

pub fn test_mirrors(mirrors: &[MirrorEntry]) -> Vec<Option<u64>> {
    mirrors.iter().map(|m| ping_host(m.ping_host)).collect()
}

pub fn recommend_index(latencies: &[Option<u64>]) -> Option<usize> {
    latencies.iter().enumerate()
        .filter_map(|(i, lat)| lat.map(|l| (i, l)))
        .min_by_key(|(_, l)| *l)
        .map(|(i, _)| i)
}

// ── 系统源切换 ───────────────────────────────────────────

const PACMAN_MIRRORLIST: &str = "/etc/pacman.d/mirrorlist";
const PACMAN_MIRRORLIST_BAK: &str = "/etc/pacman.d/mirrorlist.linux-init.bak";
const APT_SOURCES: &str = "/etc/apt/sources.list";
const APT_SOURCES_BAK: &str = "/etc/apt/sources.list.linux-init.bak";
const DNF_FEDORA_REPO: &str = "/etc/yum.repos.d/fedora.repo";
const DNF_FEDORA_REPO_BAK: &str = "/etc/yum.repos.d/fedora.repo.linux-init.bak";
const DNF_UPDATES_REPO: &str = "/etc/yum.repos.d/fedora-updates.repo";
const DNF_UPDATES_REPO_BAK: &str = "/etc/yum.repos.d/fedora-updates.repo.linux-init.bak";

fn refresh_pacman_cache() -> anyhow::Result<()> {
    let status = Command::new("sudo").args(["pacman", "-Syy"]).status()?;
    if !status.success() { anyhow::bail!("pacman -Syy 失败"); }
    Ok(())
}

fn refresh_apt_cache() -> anyhow::Result<()> {
    let status = Command::new("sudo").args(["apt", "update"]).status()?;
    if !status.success() { anyhow::bail!("apt update 失败"); }
    Ok(())
}

fn refresh_dnf_cache() -> anyhow::Result<()> {
    let status = Command::new("sudo").args(["dnf", "makecache"]).status()?;
    if !status.success() { anyhow::bail!("dnf makecache 失败"); }
    Ok(())
}

pub fn switch_pacman_mirror(url: &str) -> anyhow::Result<()> {
    if !std::path::Path::new(PACMAN_MIRRORLIST_BAK).exists() {
        let status = Command::new("sudo")
            .args(["cp", PACMAN_MIRRORLIST, PACMAN_MIRRORLIST_BAK])
            .status()?;
        if !status.success() { anyhow::bail!("备份 mirrorlist 失败"); }
    }

    if url.is_empty() {
        let status = Command::new("sudo")
            .args(["cp", PACMAN_MIRRORLIST_BAK, PACMAN_MIRRORLIST])
            .status()?;
        if !status.success() { anyhow::bail!("恢复 mirrorlist 失败"); }
    } else {
        let backup = fs::read_to_string(PACMAN_MIRRORLIST_BAK)?;
        let filtered: Vec<&str> = backup.lines()
            .filter(|l| !l.contains("## linux-init mirror"))
            .collect();
        let new_mirror = format!("## linux-init mirror\nServer = {}\n", url);
        let new_content = format!("{}\n{}", new_mirror, filtered.join("\n"));
        let tmp = "/tmp/linux-init-mirrorlist";
        fs::write(tmp, new_content)?;
        let status = Command::new("sudo").args(["cp", tmp, PACMAN_MIRRORLIST]).status()?;
        let _ = fs::remove_file(tmp);
        if !status.success() { anyhow::bail!("写入 mirrorlist 失败"); }
    }
    refresh_pacman_cache()
}

pub fn switch_apt_mirror(url: &str) -> anyhow::Result<()> {
    if !std::path::Path::new(APT_SOURCES_BAK).exists() {
        let status = Command::new("sudo")
            .args(["cp", APT_SOURCES, APT_SOURCES_BAK])
            .status()?;
        if !status.success() { anyhow::bail!("备份 sources.list 失败"); }
    }

    if url.is_empty() {
        let status = Command::new("sudo")
            .args(["cp", APT_SOURCES_BAK, APT_SOURCES])
            .status()?;
        if !status.success() { anyhow::bail!("恢复 sources.list 失败"); }
    } else {
        let backup = fs::read_to_string(APT_SOURCES_BAK)?;
        let new_content = replace_apt_mirror(&backup, url);
        let tmp = "/tmp/linux-init-sources.list";
        fs::write(tmp, new_content)?;
        let status = Command::new("sudo").args(["cp", tmp, APT_SOURCES]).status()?;
        let _ = fs::remove_file(tmp);
        if !status.success() { anyhow::bail!("写入 sources.list 失败"); }
    }
    refresh_apt_cache()
}

/// 切换 Fedora 系统源镜像（url 为镜像主机名，如 mirrors.tuna.tsinghua.edu.cn）
pub fn switch_dnf_mirror(url: &str) -> anyhow::Result<()> {
    // 备份原始 repo 文件
    for (repo, bak) in [(DNF_FEDORA_REPO, DNF_FEDORA_REPO_BAK), (DNF_UPDATES_REPO, DNF_UPDATES_REPO_BAK)] {
        if !std::path::Path::new(bak).exists() {
            let status = Command::new("sudo")
                .args(["cp", repo, bak])
                .status()?;
            if !status.success() {
                anyhow::bail!("备份 {} 失败", repo);
            }
        }
    }

    if url.is_empty() {
        // 恢复默认
        for (repo, bak) in [(DNF_FEDORA_REPO, DNF_FEDORA_REPO_BAK), (DNF_UPDATES_REPO, DNF_UPDATES_REPO_BAK)] {
            let status = Command::new("sudo")
                .args(["cp", bak, repo])
                .status()?;
            if !status.success() {
                anyhow::bail!("恢复 {} 失败", repo);
            }
        }
    } else {
        // 替换 fedora.repo
        let backup = fs::read_to_string(DNF_FEDORA_REPO_BAK)?;
        let new_content = replace_dnf_repo(&backup, "releases", url);
        let tmp = "/tmp/linux-init-fedora.repo";
        fs::write(tmp, new_content)?;
        let status = Command::new("sudo").args(["cp", tmp, DNF_FEDORA_REPO]).status()?;
        let _ = fs::remove_file(tmp);
        if !status.success() { anyhow::bail!("写入 fedora.repo 失败"); }

        // 替换 fedora-updates.repo
        let backup = fs::read_to_string(DNF_UPDATES_REPO_BAK)?;
        let new_content = replace_dnf_repo(&backup, "updates", url);
        let tmp = "/tmp/linux-init-fedora-updates.repo";
        fs::write(tmp, new_content)?;
        let status = Command::new("sudo").args(["cp", tmp, DNF_UPDATES_REPO]).status()?;
        let _ = fs::remove_file(tmp);
        if !status.success() { anyhow::bail!("写入 fedora-updates.repo 失败"); }
    }
    refresh_dnf_cache()
}

/// 替换 dnf repo 文件中的 baseurl，注释掉 metalink
fn replace_dnf_repo(content: &str, repo_type: &str, mirror_host: &str) -> String {
    content.lines().map(|line| {
        let trimmed = line.trim();
        // 注释掉 metalink 行
        if trimmed.starts_with("metalink=") {
            return format!("#{}", line);
        }
        // 替换 baseurl 行
        if trimmed.starts_with("baseurl=") {
            if let Some(idx) = trimmed.find('=') {
                let old_url = &trimmed[idx + 1..];
                let new_url = replace_dnf_mirror_host(old_url, repo_type, mirror_host);
                // 保留原始缩进
                let prefix: String = line.chars().take_while(|c| c.is_whitespace()).collect();
                return format!("{}baseurl={}", prefix, new_url);
            }
        }
        line.to_string()
    }).collect::<Vec<_>>().join("\n")
}

/// 将 baseurl 中的官方主机名替换为镜像主机名
fn replace_dnf_mirror_host(original_url: &str, repo_type: &str, mirror_host: &str) -> String {
    // 常见格式: https://download.example.com/pub/fedora/linux/releases/$releasever/Everything/$basearch/os/
    // 替换为:   https://mirrors.tuna.tsinghua.edu.cn/fedora/releases/$releasever/Everything/$basearch/os/
    if let Some(idx) = original_url.find("/fedora/linux/") {
        let path = &original_url[idx + "/fedora/linux/".len()..];
        return format!("https://{}/fedora/{}/{}", mirror_host, repo_type, path);
    }
    // 兜底：直接替换主机名部分
    if let Some(idx) = original_url.find("://") {
        let after_proto = &original_url[idx + 3..];
        if let Some(slash_idx) = after_proto.find('/') {
            let path = &after_proto[slash_idx..];
            return format!("https://{}{}", mirror_host, path);
        }
    }
    original_url.to_string()
}

fn replace_apt_mirror(content: &str, new_mirror: &str) -> String {
    let host = new_mirror.trim_start_matches("https://").trim_start_matches("http://");
    content.lines().map(|line| {
        let trimmed = line.trim();
        if trimmed.starts_with("deb ") && !trimmed.starts_with("deb-src") {
            if let Some(idx) = trimmed.find("://") {
                // 保留 [arch=amd64] 等 APT options
                let prefix = &trimmed[..idx];
                let mut options = "";
                if let Some(bracket_start) = prefix.find('[') {
                    options = &prefix[bracket_start..];
                }
                let after_proto = &trimmed[idx + 3..];
                if let Some(space_idx) = after_proto.find(char::is_whitespace) {
                    let rest = &after_proto[space_idx..];
                    if options.is_empty() {
                        return format!("deb https://{}{}", host, rest);
                    } else {
                        return format!("deb {} https://{}{}", options, host, rest);
                    }
                }
            }
        }
        line.to_string()
    }).collect::<Vec<_>>().join("\n")
}

// ── Docker 源切换 ─────────────────────────────────────────

const DOCKER_DAEMON_JSON: &str = "/etc/docker/daemon.json";

pub fn switch_docker_mirror(url: &str) -> anyhow::Result<()> {
    let mut config: serde_json::Value = if std::path::Path::new(DOCKER_DAEMON_JSON).exists() {
        let content = fs::read_to_string(DOCKER_DAEMON_JSON)?;
        serde_json::from_str(&content).unwrap_or(serde_json::json!({}))
    } else {
        serde_json::json!({})
    };

    if url.is_empty() {
        if let Some(obj) = config.as_object_mut() {
            obj.remove("registry-mirrors");
        }
    } else {
        config["registry-mirrors"] = serde_json::json!([url]);
    }

    let new_content = serde_json::to_string_pretty(&config)?;
    let tmp = "/tmp/linux-init-daemon.json";
    fs::write(tmp, new_content)?;
    let _ = Command::new("sudo").args(["mkdir", "-p", "/etc/docker"]).status();
    let status = Command::new("sudo").args(["cp", tmp, DOCKER_DAEMON_JSON]).status()?;
    let _ = fs::remove_file(tmp);
    if !status.success() { anyhow::bail!("写入 daemon.json 失败"); }
    restart_docker()
}

fn restart_docker() -> anyhow::Result<()> {
    let status = Command::new("sudo").args(["systemctl", "restart", "docker"]).status()?;
    if !status.success() { anyhow::bail!("systemctl restart docker 失败"); }
    Ok(())
}

// ── Node (npm) 源切换 ─────────────────────────────────────

pub fn switch_npm_registry(url: &str) -> anyhow::Result<()> {
    if url.is_empty() {
        anyhow::bail!("npm registry URL 不能为空");
    }
    let status = if let Ok(sudo_user) = std::env::var("SUDO_USER") {
        Command::new("sudo").args(["-u", &sudo_user, "npm", "config", "set", "registry", url]).status()?
    } else {
        Command::new("npm").args(["config", "set", "registry", url]).status()?
    };
    if !status.success() { anyhow::bail!("npm config set registry 失败"); }
    Ok(())
}

// ── 工具函数 ─────────────────────────────────────────────

pub fn system_mirrors() -> Vec<MirrorEntry> {
    match distro::detect().family() {
        DistroFamily::Arch => pacman_mirrors(),
        DistroFamily::Debian => apt_mirrors(),
        DistroFamily::Fedora => dnf_mirrors(),
        _ => vec![],
    }
}

pub fn switch_system_mirror(url: &str) -> anyhow::Result<()> {
    match distro::detect().family() {
        DistroFamily::Arch => switch_pacman_mirror(url),
        DistroFamily::Debian => switch_apt_mirror(url),
        DistroFamily::Fedora => switch_dnf_mirror(url),
        _ => anyhow::bail!("不支持的发行版"),
    }
}
