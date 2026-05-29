---
name: command-dependency-check
description: Check if external commands exist and auto-install missing dependencies in Rust CLI tools
source: auto-skill
extracted_at: '2026-05-29T09:16:26.798Z'
---

# Command Dependency Checking and Auto-Installation

## When to use
When building Rust CLI tools that invoke external commands (curl, git, etc.) and need to:
- Verify commands exist before using them
- Provide helpful error messages when commands are missing
- Auto-install missing commands when possible
- Minimize dependencies (only install when actually needed)

## Core Pattern

Create a `utils.rs` module with two functions:

```rust
use std::process::Command;

/// Check if a command exists in PATH
pub fn command_exists(cmd: &str) -> bool {
    Command::new("which")
        .arg(cmd)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Ensure a command exists, installing it if necessary
pub fn ensure_command(cmd: &str) -> anyhow::Result<()> {
    if command_exists(cmd) {
        return Ok(());
    }

    // Map command names to package names (they may differ)
    let package = match cmd {
        "curl" => "curl",
        "git" => "git",
        "vim" => "vim",
        // Add more mappings as needed
        _ => cmd,  // Default: command name == package name
    };

    crate::distro::install_packages(&[package])
}
```

## Usage Pattern

Call `ensure_command()` at the start of operations that need external tools:

```rust
pub fn install_oh_my_zsh() -> anyhow::Result<()> {
    // Ensure dependencies exist before proceeding
    crate::utils::ensure_command("curl")?;
    crate::utils::ensure_command("git")?;
    
    // Now safe to use curl and git
    let download = Command::new("curl")
        .args(["-fSL", "-o", tmp_script, url])
        .status()?;
    // ...
}

pub fn install_vundle() -> anyhow::Result<()> {
    crate::utils::ensure_command("git")?;
    
    let status = Command::new("git")
        .args(["clone", repo_url, vundle_dir])
        .status()?;
    // ...
}
```

## Key Design Principles

1. **Lazy installation**: Only check/install commands when they're actually needed
   - Don't install curl at startup if user never chooses oh-my-zsh
   - Each module is responsible for its own dependencies

2. **Command vs package name mapping**: Some commands have different package names
   - `fd` command → `fd` package (Arch) or `fd-find` package (Debian)
   - `docker-compose` command → `docker-compose` (Arch) or `docker-compose-v2` (Debian)
   - The `ensure_command` function should map command names to package names

3. **Fail gracefully**: If installation fails, propagate the error with context
   ```rust
   pub fn ensure_command(cmd: &str) -> anyhow::Result<()> {
       if command_exists(cmd) {
           return Ok(());
       }
       
       let package = map_command_to_package(cmd);
       crate::distro::install_packages(&[package])
           .with_context(|| format!("Failed to install required command: {}", cmd))
   }
   ```

4. **Idempotent**: `ensure_command` can be called multiple times safely
   - If command exists, returns immediately (no overhead)
   - If command missing, installs it once
   - Subsequent calls find the command and return

## System Commands (No Check Needed)

These commands are part of base Linux systems and don't need checking:
- `sudo`, `sh`, `bash` — always present
- `chown`, `getent` — coreutils/base packages
- `systemctl` — systemd (universal on target distros)
- `groups`, `locale` — standard utilities
- `dpkg`, `pacman` — package managers (detected via distro family)
- `ssh-keygen`, `which`, `chsh` — standard tools

## Commands That Need Checking

Only check commands that:
- Are not part of base system
- Might not be installed by default
- Are needed for specific features

Examples:
- `curl` — needed for downloads (oh-my-zsh, nvm)
- `git` — needed for cloning plugins (vundle, oh-my-zsh plugins)
- `vim` — needed for vim plugin installation (vundle requires vim to be running)

## Integration with Multi-Distro Support

The `ensure_command` pattern integrates with the distro abstraction layer:

```rust
// In utils.rs
pub fn ensure_command(cmd: &str) -> anyhow::Result<()> {
    if command_exists(cmd) {
        return Ok(());
    }

    let package = match cmd {
        "curl" => "curl",
        "git" => "git",
        _ => cmd,
    };

    // Use the distro layer's install_packages function
    // which handles pacman vs apt automatically
    crate::distro::install_packages(&[package])
}
```

This way, `ensure_command("git")` works on both Arch (calls `pacman -S git`) and Debian (calls `apt install git`) without the caller needing to know which distro they're on.

## Alternative: Return Command Path

For commands that might be in non-standard locations, you can return the full path:

```rust
pub fn ensure_command_path(cmd: &str) -> anyhow::Result<PathBuf> {
    if let Ok(output) = Command::new("which").arg(cmd).output() {
        if output.status.success() {
            let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
            return Ok(PathBuf::from(path));
        }
    }
    
    // Command not found, install it
    ensure_command(cmd)?;
    
    // Try again
    let output = Command::new("which")
        .arg(cmd)
        .output()
        .with_context(|| format!("Command {} still not found after installation", cmd))?;
    
    let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
    Ok(PathBuf::from(path))
}
```

Use this when you need to invoke commands via full path (e.g., for security or reliability).

## Gotchas

1. **`which` command availability**: The `which` command itself is part of coreutils and always available on Linux, so it's safe to use for checking.

2. **Package manager prompts**: When `ensure_command` triggers installation, the package manager might prompt for confirmation. Ensure your installation function uses non-interactive flags (`--noconfirm` for pacman, `-y` for apt).

3. **Sudo requirements**: Installing packages requires sudo. If your tool runs without sudo initially, `ensure_command` will fail when it tries to install. Either:
   - Run the entire tool with sudo (recommended for system setup tools)
   - Check for sudo early and warn the user
   - Skip auto-install and just report the missing command

4. **Network failures**: If `ensure_command` tries to install a package and network is unavailable, it will fail. The error will propagate naturally, but you might want to add context:
   ```rust
   crate::distro::install_packages(&[package])
       .with_context(|| format!("Failed to install {}. Check your internet connection.", cmd))
   ```

5. **Command name vs binary name**: Sometimes the command you invoke differs from the package name:
   - Package `fd` installs binary `fd` (Arch) or `fdfind` (Debian)
   - For these cases, you might need distro-specific command checking
