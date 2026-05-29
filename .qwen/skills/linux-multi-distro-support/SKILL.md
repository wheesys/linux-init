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
| SSH server | `openssh` | `openssh-server` |
| vim | `vim` | `vim` |

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

## SSH Server Configuration

Package names differ (`openssh` vs `openssh-server`), and `sshd_config` editing requires `sudo tee`:

```rust
fn disable_root_login() -> anyhow::Result<()> {
    let content = fs::read_to_string("/etc/ssh/sshd_config")?;
    let mut new_content = String::new();
    let mut found = false;

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("PermitRootLogin") || trimmed.starts_with("#PermitRootLogin") {
            new_content.push_str("PermitRootLogin no\n");
            found = true;
        } else {
            new_content.push_str(line);
            new_content.push('\n');
        }
    }
    if !found {
        new_content.push_str("\nPermitRootLogin no\n");
    }

    // Write via sudo tee (file is root-owned)
    let mut child = Command::new("sudo")
        .args(["tee", "/etc/ssh/sshd_config"])
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .spawn()?;
    if let Some(ref mut stdin) = child.stdin {
        stdin.write_all(new_content.as_bytes())?;
    }
    drop(child.stdin.take());
    child.wait()?;
    Ok(())
}
```

Check if already configured:
```rust
fn is_root_login_disabled() -> bool {
    fs::read_to_string("/etc/ssh/sshd_config")
        .map(|c| c.lines().any(|l| l.trim() == "PermitRootLogin no"))
        .unwrap_or(false)
}
```

Start service (identical on both families):
```rust
Command::new("sudo")
    .args(["systemctl", "enable", "--now", "sshd"])
    .status()?;
```

## Vim + Vundle Plugin Manager

Vim package name is `vim` on both families. Vundle install via git clone:

```rust
let home = dirs::home_dir()?;
let vundle_dir = home.join(".vim/bundle/Vundle.vim");
if !vundle_dir.exists() {
    Command::new("git")
        .args(["clone", "https://github.com/VundleVim/Vundle.vim.git", vundle_dir.to_str()?])
        .status()?;
}
```

Write `.vimrc` with Vundle plugin declarations:
```
set nocompatible
filetype off
set rtp+=~/.vim/bundle/Vundle.vim
call vundle#begin()
Plugin 'VundleVim/Vundle.vim'
Plugin 'preservim/nerdtree'
...
call vundle#end()
filetype plugin indent on
```

## Common Patterns Across Distros

These work identically on all target distros:
- `systemctl enable --now <service>` (all use systemd)
- `usermod -aG docker $USER` (same on both families)
- `chsh -s $(which zsh)` (same on both)
- `ssh-keygen` (same on both)
- `locale-gen` (same on both, after editing locale.gen)
