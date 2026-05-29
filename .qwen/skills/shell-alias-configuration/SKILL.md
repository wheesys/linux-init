---
name: shell-alias-configuration
description: Automatically configure shell aliases and hooks in .bashrc/.zshrc with marker-based deduplication
source: auto-skill
extracted_at: '2026-05-29T12:15:23.208Z'
---

# Shell Alias and Hook Configuration Pattern

## When to use
When building tools that need to:
- Configure command aliases after installing replacement tools
- Add shell hooks for tools like direnv, nvm, or other shell-integrated utilities
- Automatically enhance shell environments with modern CLI tool alternatives

## Core Pattern

### 1. Marker-Based Deduplication

Use comment markers to avoid adding the same configuration multiple times:

```rust
fn append_aliases_to_file(path: &Path, aliases: &[&str], shell: &str) -> anyhow::Result<()> {
    let content = fs::read_to_string(path)?;
    
    // Check if already configured using a marker
    let marker = format!("# Linux Init - {} aliases", shell);
    if content.contains(&marker) {
        return Ok(());  // Already configured, skip
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
```

**Key points:**
- Use a unique marker comment per configuration section
- Check for marker existence before writing
- Add a blank line before the marker for readability
- Write marker first, then configuration lines

### 2. Multi-Shell Support

Configure both bash and zsh if they exist:

```rust
pub fn configure_aliases(installed_tools: &[&str]) -> anyhow::Result<()> {
    let home = get_real_home()?;
    
    // Build alias list based on installed tools
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
    
    // Configure bash if it exists
    let bashrc_path = home.join(".bashrc");
    if bashrc_path.exists() {
        append_aliases_to_file(&bashrc_path, &aliases, "bash")?;
    }
    
    // Configure zsh if it exists
    let zshrc_path = home.join(".zshrc");
    if zshrc_path.exists() {
        append_aliases_to_file(&zshrc_path, &aliases, "zsh")?;
    }
    
    Ok(())
}
```

### 3. Shell Hook Configuration (direnv Example)

For tools that require shell hooks (like direnv), use a similar pattern:

```rust
pub fn configure_direnv_hook() -> anyhow::Result<()> {
    let home = get_real_home()?;
    
    // Verify tool is installed
    if !distro::is_package_installed(distro::package_name("direnv").unwrap_or("direnv")) {
        return Ok(());
    }
    
    // Configure bash
    let bashrc_path = home.join(".bashrc");
    if bashrc_path.exists() {
        append_direnv_hook_to_file(&bashrc_path, "bash")?;
    }
    
    // Configure zsh
    let zshrc_path = home.join(".zshrc");
    if zshrc_path.exists() {
        append_direnv_hook_to_file(&zshrc_path, "zsh")?;
    }
    
    Ok(())
}

fn append_direnv_hook_to_file(path: &Path, shell: &str) -> anyhow::Result<()> {
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
```

**Note:** direnv hook syntax differs between shells:
- bash: `eval "$(direnv hook bash)"`
- zsh: `eval "$(direnv hook zsh)"`

### 4. Integration with Tool Installation

Call configuration functions after installing tools:

```rust
run_in_terminal(terminal, || {
    // Install tools
    let refs: Vec<&str> = selected_owned.iter().map(|s| s.as_str()).collect();
    crate::modules::tools::install_tools(&refs)?;
    
    // Configure aliases for installed tools
    crate::modules::tools::configure_aliases(&refs)?;
    
    // Configure direnv hook if installed
    if refs.contains(&"direnv") {
        crate::modules::tools::configure_direnv_hook()?;
    }
    
    Ok(())
})?;
```

## Common Modern CLI Tool Aliases

| Traditional Tool | Modern Alternative | Alias |
|-----------------|-------------------|-------|
| `rm` | `trash-cli` | `alias rm='trash '` |
| `ps` | `procs` | `alias ps='procs '` |
| `du` | `dust` | `alias du='dust '` |
| `df` | `duf` | `alias df='duf '` |
| `ls -lah` | `eza -lah` | `alias la='eza -lah'` |

**Important:** Use trailing space in aliases like `alias rm='trash '` to allow additional arguments to pass through.

## Sudo User Home Directory Detection

When running with sudo, use `get_real_home()` to get the actual user's home directory:

```rust
fn get_real_home() -> anyhow::Result<std::path::PathBuf> {
    if let Ok(sudo_user) = std::env::var("SUDO_USER") {
        let output = Command::new("getent")
            .args(["passwd", &sudo_user])
            .output()?;
        let line = String::from_utf8_lossy(&output.stdout);
        if let Some(home) = line.split(':').nth(5) {
            return Ok(std::path::PathBuf::from(home.trim()));
        }
    }
    dirs::home_dir().ok_or_else(|| anyhow::anyhow!("无法获取 home 目录"))
}
```

This ensures aliases are written to the real user's shell config files, not root's.

## User Experience Considerations

1. **Inform users about configuration:**
   ```rust
   Ok(match lang {
       Lang::Chinese => format!("✅ {} 个工具安装成功，别名已配置", count),
       Lang::English => format!("✅ {} tools installed, aliases configured", count),
   })
   ```

2. **Remind users to reload shell:**
   After configuration, users need to either:
   - Open a new terminal, or
   - Run `source ~/.bashrc` or `source ~/.zshrc`

3. **Idempotent operations:**
   The marker-based approach ensures running the tool multiple times doesn't duplicate configuration.

## Complete Example

```rust
use crate::distro;
use crate::utils::get_real_home;
use std::fs;
use std::io::Write;
use std::path::Path;

/// Configure command aliases for modern CLI tools
pub fn configure_aliases(installed_tools: &[&str]) -> anyhow::Result<()> {
    let home = get_real_home()?;
    
    let mut aliases = Vec::new();
    
    // Build alias list based on what was installed
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
    
    // Apply to existing shell configs
    let bashrc_path = home.join(".bashrc");
    if bashrc_path.exists() {
        append_aliases_to_file(&bashrc_path, &aliases, "bash")?;
    }
    
    let zshrc_path = home.join(".zshrc");
    if zshrc_path.exists() {
        append_aliases_to_file(&zshrc_path, &aliases, "zsh")?;
    }
    
    Ok(())
}

fn append_aliases_to_file(path: &Path, aliases: &[&str], shell: &str) -> anyhow::Result<()> {
    let content = fs::read_to_string(path)?;
    
    let marker = format!("# Linux Init - {} aliases", shell);
    if content.contains(&marker) {
        return Ok(());  // Idempotent: already configured
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

/// Configure direnv shell hook
pub fn configure_direnv_hook() -> anyhow::Result<()> {
    let home = get_real_home()?;
    
    // Verify direnv is installed
    if !distro::is_package_installed(distro::package_name("direnv").unwrap_or("direnv")) {
        return Ok(());
    }
    
    let bashrc_path = home.join(".bashrc");
    if bashrc_path.exists() {
        append_direnv_hook_to_file(&bashrc_path, "bash")?;
    }
    
    let zshrc_path = home.join(".zshrc");
    if zshrc_path.exists() {
        append_direnv_hook_to_file(&zshrc_path, "zsh")?;
    }
    
    Ok(())
}

fn append_direnv_hook_to_file(path: &Path, shell: &str) -> anyhow::Result<()> {
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
```

## Benefits

1. **Automatic enhancement:** Users get modern CLI tools configured without manual editing
2. **Safe and idempotent:** Marker-based deduplication prevents configuration duplication
3. **Multi-shell support:** Works with both bash and zsh automatically
4. **Transparent:** Users can see exactly what was added to their shell configs
5. **Reversible:** Users can easily remove the section by deleting the marker and following lines
