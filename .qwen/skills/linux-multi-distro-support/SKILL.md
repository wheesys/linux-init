---
name: linux-multi-distro-support
description: Detect Linux distro via /etc/os-release and abstract package management across Arch(pacman) and Debian(apt) families
source: auto-skill
extracted_at: '2026-05-29T06:57:19.446Z'
---

# Multi-Distro Linux Package Management Abstraction

## When to use
When building tools that need to support multiple Linux distributions with different package managers.

## Distro Detection via /etc/os-release

All modern Linux distros (freedesktop standard) provide `/etc/os-release`. Parse key fields:

```rust
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
```

Key fields:
- `ID` — exact distro identifier: `arch`, `ubuntu`, `debian`, `cachyos`, `manjaro`
- `ID_LIKE` — parent/family: `arch` for Arch derivatives, `debian` for Debian derivatives
- `VERSION_ID` — version string like `24.04` or `12`

### Detection logic

```rust
enum DistroFamily { Arch, Debian, Unknown }

fn detect_distro() -> Distro {
    // 1. Check ID for exact match
    // 2. Check ID_LIKE for family match (contains "arch" or "debian")
    // 3. Fall back to Unknown
}
```

**Key insight**: Arch/CachyOS/Manjaro all use pacman; Ubuntu/Debian both use apt. So 5 distros = only 2 package managers.

## Package Name Mapping

Different distros use different package names for the same tool:

| Tool | Arch (pacman) | Debian/Ubuntu (apt) |
|------|---------------|---------------------|
| fd | `fd` | `fd-find` |
| docker-compose | `docker-compose` | `docker-compose-v2` |
| CJK fonts | `noto-fonts-cjk` | `fonts-noto-cjk` |
| docker | `docker` | `docker.io` |

Maintain a mapping function per distro family:

```rust
pub fn package_name(tool: &str) -> Option<&'static str> {
    match tool {
        "fd" => Some("fd-find"),  // Debian name
        "docker" => Some("docker.io"),
        // ...
        _ => None,
    }
}
```

## Package Installation

### pacman (Arch family)
```rust
Command::new("sudo")
    .arg("pacman")
    .arg("-S")
    .arg("--noconfirm")
    .arg("--needed")  // skip already installed
    .args(packages)
    .status()?;
```

### apt (Debian family)
```rust
// Always update first
Command::new("sudo").arg("apt").arg("update").status()?;
Command::new("sudo")
    .arg("apt")
    .arg("install")
    .arg("-y")
    .args(packages)
    .status()?;
```

## Package Installed Check

```rust
// Arch
Command::new("pacman").arg("-Qi").arg(pkg).output()
    .map(|o| o.status.success()).unwrap_or(false)

// Debian
Command::new("dpkg").arg("-s").arg(pkg).output()
    .map(|o| o.status.success()).unwrap_or(false)
```

## Writing to /etc/ files (e.g., locale.gen)

Files under `/etc/` are root-owned. Use `sudo tee` pattern:

```rust
let mut child = Command::new("sudo")
    .args(["tee", "/etc/locale.gen"])
    .stdin(Stdio::piped())
    .stdout(Stdio::null())
    .spawn()?;

if let Some(ref mut stdin) = child.stdin {
    stdin.write_all(content.as_bytes())?;
}
drop(child.stdin.take());
child.wait()?;
```

## Common Patterns Across Distros

These work identically on all target distros:
- `systemctl enable --now <service>` (all use systemd)
- `usermod -aG docker $USER` (same on both families)
- `chsh -s $(which zsh)` (same on both)
- `ssh-keygen` (same on both)
- `locale-gen` (same on both, after editing locale.gen)
