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
        _ => {
            if id_like.contains("arch") {
                Distro::Arch
            } else if id_like.contains("debian") || id_like.contains("ubuntu") {
                Distro::Debian(version_id)
            } else {
                Distro::Unknown(id.to_string())
            }
        }
    }
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
