use super::Distro;
use std::collections::HashMap;
use std::fs;

pub fn detect_distro() -> Distro {
    let os_release = match fs::read_to_string("/etc/os-release") {
        Ok(content) => content,
        Err(_) => return Distro::Unknown("无法读取 /etc/os-release".to_string()),
    };

    let fields = parse_os_release(&os_release);

    let id = fields.get("ID").map(|s| s.as_str()).unwrap_or("");
    let id_like = fields.get("ID_LIKE").map(|s| s.as_str()).unwrap_or("");
    let version_id = fields
        .get("VERSION_ID")
        .map(|s| s.as_str())
        .unwrap_or("")
        .to_string();

    match id {
        "arch" => Distro::Arch,
        "cachyos" => Distro::CachyOS,
        "manjaro" => Distro::Manjaro,
        "ubuntu" => Distro::Ubuntu(version_id),
        "debian" => Distro::Debian(version_id),
        "fedora" => {
            // Fedora Silverblue/Kinoite 等 ostree 变体不支持
            let variant_id = fields
                .get("VARIANT_ID")
                .map(|s| s.as_str())
                .unwrap_or("");
            if matches!(variant_id, "silverblue" | "kinoite" | "sericea" | "onyx") {
                return Distro::Unknown(format!("Fedora {}（ostree 变体不支持）", variant_id));
            }
            Distro::Fedora(version_id)
        }
        _ => {
            if id_like.contains("arch") {
                Distro::Arch
            } else if id_like.contains("debian") || id_like.contains("ubuntu") {
                Distro::Debian(version_id)
            } else if id_like.contains("fedora") || id_like.contains("rhel") {
                Distro::Fedora(version_id)
            } else {
                Distro::Unknown(id.to_string())
            }
        }
    }
}

/// 检测是否在 WSL 环境中运行（支持 WSL2）
pub fn is_wsl() -> bool {
    // /proc/sys/fs/binfmt_misc/WSLInterop 存在于 WSL1 和 WSL2
    if std::path::Path::new("/proc/sys/fs/binfmt_misc/WSLInterop").exists() {
        return true;
    }
    // Fallback: 检查 /proc/version 中的 Microsoft/WSL 标记
    std::fs::read_to_string("/proc/version")
        .map(|s| s.to_lowercase().contains("microsoft") || s.contains("WSL"))
        .unwrap_or(false)
}

fn parse_os_release(content: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    for line in content.lines() {
        if let Some((key, value)) = line.split_once('=') {
            let value = value.trim_matches('"').trim_matches('\'');
            map.insert(key.to_string(), value.to_string());
        }
    }
    map
}
