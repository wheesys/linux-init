---
name: rust-ratatui-tui-app
description: Scaffold a Rust TUI app with ratatui+crossterm including event loop, menu navigation, and interactive command execution pattern
source: auto-skill
extracted_at: '2026-05-29T06:57:19.446Z'
---

# Rust ratatui TUI Application Pattern

## When to use
When building a terminal UI (TUI) application in Rust that needs:
- Interactive menus with keyboard navigation
- Running shell commands (especially `sudo` commands that need user interaction)
- CJK/Unicode text support
- Clean state management

## Dependencies (Cargo.toml)

```toml
[dependencies]
ratatui = "0.29"
crossterm = "0.28"
anyhow = "1"

[profile.release]
strip = true
lto = true
codegen-units = 1
```

## Core Architecture

### 1. Terminal Setup / Teardown

```rust
use ratatui::{backend::CrosstermBackend, Terminal};
use crossterm::{event, execute, terminal};
use std::io::{self, Stdout};

pub type Term = Terminal<CrosstermBackend<Stdout>>;

pub fn setup_terminal() -> io::Result<Term> {
    terminal::enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, terminal::EnterAlternateScreen, crossterm::cursor::Hide)?;
    Terminal::new(CrosstermBackend::new(stdout))
}

pub fn restore_terminal(terminal: &mut Term) -> io::Result<()> {
    terminal::disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        crossterm::cursor::Show,
        terminal::LeaveAlternateScreen
    )?;
    Ok(())
}
```

### 2. Run Interactive Commands (Critical Pattern)

When you need to run commands that require user interaction (e.g., `sudo` asking for password, `chsh`, `ssh-keygen`), you MUST temporarily leave the TUI alternate screen:

```rust
fn run_in_terminal<F: FnOnce() -> anyhow::Result<()>>(
    terminal: &mut Term,
    f: F,
) -> anyhow::Result<()> {
    // Step 1: Leave alternate screen, restore cursor
    terminal::disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        crossterm::cursor::Show,
        terminal::LeaveAlternateScreen
    )?;

    // Step 2: Run the interactive command
    let result = f();

    // Step 3: Re-enter alternate screen, re-enable raw mode
    terminal::enable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        crossterm::cursor::Hide,
        terminal::EnterAlternateScreen
    )?;
    terminal.clear()?;

    result
}
```

The command itself uses `Stdio::inherit()` to pass through stdin/stdout/stderr:

```rust
let status = Command::new("sudo")
    .args(["pacman", "-S", "--noconfirm", "zsh"])
    .stdin(Stdio::inherit())
    .stdout(Stdio::inherit())
    .stderr(Stdio::inherit())
    .status()?;
```

### 3. Event Loop

```rust
fn run_app(terminal: &mut Term, app: &mut App) -> Result<()> {
    while app.running {
        terminal.draw(|frame| ui::render(frame, app))?;

        // Poll with timeout to allow UI refresh
        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                handle_key(app, key);
            }
        }
    }
    Ok(())
}
```

### 4. Menu Navigation State Machine

Use an enum for pages + index for cursor position:

```rust
pub enum Page {
    MainMenu,
    Shell,
    Docker,
    // ...
}

pub struct App {
    pub page: Page,
    pub menu_index: usize,  // cursor position
    // ...
}
```

Key handling pattern for each page:

```rust
fn handle_main_menu(app: &mut App, key: KeyEvent) {
    let max = MENU_ITEMS.len();
    match key.code {
        KeyCode::Up => app.menu_index = app.menu_index.saturating_sub(1),
        KeyCode::Down => app.menu_index = (app.menu_index + 1).min(max - 1),
        KeyCode::Enter => { /* navigate to sub-page */ },
        KeyCode::Esc => { /* go back */ },
        KeyCode::Char('q') => { app.running = false; },
        _ => {}
    }
}
```

### 5. Multi-select Pattern (Space to toggle)

```rust
pub struct App {
    pub selected_items: Vec<bool>,  // parallel to items list
    pub cursor_index: usize,
}

// Key handler:
KeyCode::Char(' ') => {
    if let Some(val) = app.selected_items.get_mut(app.cursor_index) {
        *val = !*val;
    }
    // Auto-advance cursor
    app.cursor_index = (app.cursor_index + 1).min(max - 1);
}
```

### 6. Rendering with Status Indicators

Show ✓ for completed/installed items, ▸ for current selection:

```rust
let marker = if i == selected {
    Span::styled("▸ ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
} else {
    Span::raw("  ")
};
let check = if installed {
    Span::styled("✓ ", Style::default().fg(Color::Green))
} else {
    Span::styled("  ", Style::default().fg(Color::DarkGray))
};
```

### 7. Status Bar at Bottom

Reserve 1 line for keyboard hints:

```rust
let chunks = Layout::default()
    .direction(Direction::Vertical)
    .constraints([Constraint::Min(3), Constraint::Length(1)])
    .split(frame.area());
// chunks[0] = main content, chunks[1] = status bar
```

## i18n Pattern (Zero-Dependency)

For TUI apps supporting multiple languages, use a dedicated `i18n.rs` module with match-on-lang functions:

```rust
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Lang { Chinese, English }

// For static strings:
pub fn main_title(lang: Lang) -> &'static str {
    match lang {
        Lang::Chinese => "环境初始化向导",
        Lang::English => "Environment Setup Wizard",
    }
}

// For dynamic strings (format!):
pub fn msg_success(lang: Lang, name: &str) -> String {
    match lang {
        Lang::Chinese => format!("✅ {} 安装成功", name),
        Lang::English => format!("✅ {} installed successfully", name),
    }
}

// For descriptions indexed by name (plugins, tools, themes):
pub fn plugin_desc(lang: Lang, name: &str) -> &'static str {
    match (lang, name) {
        (Lang::Chinese, "git") => "Git 别名和补全",
        (Lang::English, "git") => "Git aliases and completions",
        _ => "",
    }
}
```

**Key rules:**
- Keep ALL display strings in `i18n.rs` — never hardcode Chinese/English in UI code
- Use `&'static str` for constant strings, `String` only when `format!` is needed
- Functions returning `&str` from a `kind`/`category` parameter must return `String` (not `&'static str`) because the fallback `_ => input` can't produce a `'static` reference
- Detect default language from `$LANG` env var at startup: `if lang.starts_with("zh") { Chinese } else { English }`
- Add a `LangSelect` page as the first screen before MainMenu

## Terminal Theme Compatibility

**Always use `Color::Reset` instead of `Color::White`** for text that should be visible on both light and dark terminal backgrounds:

```rust
// WRONG — invisible on white backgrounds
Span::styled(name, Style::default().fg(Color::White))

// CORRECT — uses terminal's default foreground color
Span::styled(name, Style::default().fg(Color::Reset))
```

For selected/highlighted items, combine with modifiers:
```rust
Style::default().fg(Color::Reset).add_modifier(Modifier::BOLD)
```

For list selection highlighting, use `Modifier::REVERSED` instead of explicit background colors — it swaps foreground and background, working correctly on any terminal theme:

```rust
// WRONG — DarkGray bg invisible on dark terminals, or text invisible on light terminals
.highlight_style(Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD))

// CORRECT — REVERSED swaps fg/bg, always visible regardless of terminal theme
.highlight_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD | Modifier::REVERSED))
```

## Universal Exit Handling (Ctrl+C)

TUI apps MUST support Ctrl+C as a universal exit method (terminal convention). Check for it BEFORE any other key handling:

```rust
use crossterm::event::{KeyModifiers, KeyCode};

pub fn handle_event(terminal: &mut Term, app: &mut App) -> anyhow::Result<Option<Action>> {
    if !event::poll(std::time::Duration::from_millis(50))? {
        return Ok(None);
    }
    let ev = event::read()?;
    if let Event::Key(key) = ev {
        // Ctrl+C ALWAYS exits (terminal convention)
        if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
            return Ok(Some(Action::Quit));
        }
        // q exits only on certain pages (not multi-select pages)
        if key.code == KeyCode::Char('q')
            && !matches!(app.page, Page::PluginSelect | Page::ToolSelect | ...)
        {
            return Ok(Some(Action::Quit));
        }
        return handle_key(terminal, app, key);
    }
    Ok(None)
}
```

**Key points:**
- Ctrl+C must be checked FIRST, before any page-specific handling
- Exclude multi-select pages from 'q' quit (users might type 'q' accidentally)
- When Ctrl+C exits mid-operation, do NOT save incomplete config — only save when user explicitly confirms (Enter or Esc on multi-select)

## Partial Save Behavior

For multi-step configuration (e.g., plugin selection), implement partial save:
- **Esc on multi-select page**: Save current selections and return to parent menu
- **Enter on multi-select page**: Save current selections and return to parent menu
- **Ctrl+C anywhere**: Exit immediately WITHOUT saving incomplete state

```rust
fn handle_plugins(app: &mut App, key: KeyEvent) -> anyhow::Result<Option<Action>> {
    match key.code {
        KeyCode::Esc | KeyCode::Backspace => {
            // Save on Esc (user finished selecting)
            if !app.selected_plugins.is_empty() {
                crate::modules::shell::set_plugins(&app.selected_plugins)?;
            }
            app.page = Page::Shell;
        }
        KeyCode::Enter => {
            // Save on Enter (user confirmed)
            if !app.selected_plugins.is_empty() {
                crate::modules::shell::set_plugins(&app.selected_plugins)?;
            }
            app.page = Page::Shell;
        }
        KeyCode::Char(' ') => { /* toggle selection, don't save yet */ }
        _ => {}
    }
    Ok(None)
}
```

## Config Persistence Pattern

For TUI apps that guide users through multi-step setup, persist state to avoid repeating completed steps:

### Dependencies
```toml
[dependencies]
serde = { version = "1", features = ["derive"] }
serde_json = "1"
```

### Config Structure
```rust
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    pub language: Option<String>,  // "zh" or "en"
    pub completed: CompletedModules,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CompletedModules {
    pub zsh_installed: bool,
    pub docker_installed: bool,
    pub ssh_key_generated: bool,
    // ... track each module
}
```

### Load/Save Functions
```rust
impl Config {
    pub fn load() -> Self {
        if let Some(path) = Self::config_path() {
            if path.exists() {
                if let Ok(content) = fs::read_to_string(&path) {
                    if let Ok(config) = serde_json::from_str(&content) {
                        return config;
                    }
                }
            }
        }
        Self::default()
    }

    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(path) = Self::config_path() {
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)?;
            }
            let content = serde_json::to_string_pretty(self)?;
            fs::write(&path, content)?;
        }
        Ok(())
    }

    fn config_path() -> Option<PathBuf> {
        dirs::config_dir().map(|p| p.join("your-app").join("config.json"))
    }
}
```

**IMPORTANT**: When the TUI app runs with `sudo`, `dirs::config_dir()` and `dirs::home_dir()` return root's paths. Use this sudo-aware version instead:

```rust
fn get_real_home() -> std::path::PathBuf {
    if let Ok(sudo_user) = std::env::var("SUDO_USER") {
        if let Ok(output) = std::process::Command::new("getent")
            .args(["passwd", &sudo_user])
            .output()
        {
            let line = String::from_utf8_lossy(&output.stdout);
            if let Some(home) = line.split(':').nth(5) {
                return std::path::PathBuf::from(home.trim());
            }
        }
    }
    dirs::home_dir().unwrap_or_default()
}

fn config_path() -> Option<PathBuf> {
    Some(get_real_home().join(".config").join("your-app").join("config.json"))
}
```

Also fix file/directory ownership after saving when running with sudo:
```rust
pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(path) = Self::config_path() {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
            if let Ok(sudo_user) = std::env::var("SUDO_USER") {
                let _ = std::process::Command::new("chown")
                    .args([&format!("{}:{}", sudo_user, sudo_user), parent.to_str().unwrap_or("")])
                    .status();
            }
        }
        fs::write(&path, serde_json::to_string_pretty(self)?)?;
        if let Ok(sudo_user) = std::env::var("SUDO_USER") {
            let _ = std::process::Command::new("chown")
                .args([&format!("{}:{}", sudo_user, sudo_user), path.to_str().unwrap_or("")])
                .status();
        }
    }
    Ok(())
}
```

### Skip Steps Based on Config
```rust
pub fn new(distro: Distro) -> Self {
    let config = Config::load();
    
    // Skip language selection if already saved
    let (lang, page) = if let Some(ref lang_str) = config.language {
        let lang = match lang_str.as_str() {
            "zh" => Lang::Chinese,
            _ => Lang::English,
        };
        (lang, Page::MainMenu)  // Skip to main menu
    } else {
        (Lang::Chinese, Page::LangSelect)  // Ask for language
    };
    
    Self { config, lang, page, ... }
}
```

### Save After User Actions
```rust
fn handle_lang_select(app: &mut App, key: KeyEvent) -> anyhow::Result<Option<Action>> {
    match key.code {
        KeyCode::Enter => {
            app.lang = if app.lang_index == 0 { Lang::Chinese } else { Lang::English };
            app.page = Page::MainMenu;
            
            // Save language to config
            app.config.language = Some(match app.lang {
                Lang::Chinese => "zh".to_string(),
                Lang::English => "en".to_string(),
            });
            let _ = app.config.save();
        }
        _ => {}
    }
    Ok(None)
}
```

### State Synchronization

After detecting actual system state, sync it back to config to track completions:

```rust
fn refresh_state(app: &mut App) {
    // Detect actual state
    app.zsh_installed = distro::is_package_installed("zsh");
    app.docker_installed = distro::is_package_installed("docker");
    app.ssh_key_exists = modules::ssh::has_ssh_key();
    // ...

    // Sync config with actual state
    let mut config_changed = false;

    if app.zsh_installed && !app.config.completed.zsh_installed {
        app.config.completed.zsh_installed = true;
        config_changed = true;
    }
    if app.docker_installed && !app.config.completed.docker_installed {
        app.config.completed.docker_installed = true;
        config_changed = true;
    }
    // ... for each module

    if config_changed {
        let _ = app.config.save();
    }
}
```

Call `refresh_state` after each action and on app startup to keep config in sync with reality.

### Show Completion Status in Menu
```rust
fn render_main_menu(frame: &mut Frame, app: &App, area: Rect) {
    let completion_status = vec![
        app.config.completed.zsh_installed,
        app.config.completed.docker_installed,
        app.config.completed.ssh_key_generated,
        // ...
    ];

    let items: Vec<ListItem> = menu
        .iter()
        .enumerate()
        .map(|(i, (name, desc))| {
            let status_mark = if completion_status.get(i).copied().unwrap_or(false) {
                Span::styled("✓ ", Style::default().fg(Color::Green))
            } else {
                Span::raw("  ")
            };
            // ... rest of rendering
            ListItem::new(Line::from(vec![marker, status_mark, name_s, desc_s]))
        })
        .collect();
    // ...
}
```

## Oh-My-Zsh Installation Pattern

The common curl-pipe pattern can fail due to shell expansion issues. Use download-then-execute instead:

```rust
pub fn install_oh_my_zsh() -> anyhow::Result<()> {
    let home = get_real_home()?;  // NOT dirs::home_dir() — must use real user's home under sudo
    let omz_dir = home.join(".oh-my-zsh");
    if omz_dir.exists() {
        return Ok(());
    }

    // Download install script to temp file
    let tmp_script = "/tmp/linux-init-omz-install.sh";
    let download = Command::new("curl")
        .args([
            "-fsSL", "-o", tmp_script,
            "https://raw.githubusercontent.com/ohmyzsh/ohmyzsh/master/tools/install.sh"
        ])
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()?;

    if !download.success() {
        anyhow::bail!("Oh My Zsh 安装脚本下载失败，请检查网络连接");
    }

    // CRITICAL: Run as real user (not root) when running with sudo
    let status = if let Ok(sudo_user) = std::env::var("SUDO_USER") {
        Command::new("sudo")
            .args(["-u", &sudo_user, "sh", tmp_script, "", "--unattended"])
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()?
    } else {
        Command::new("sh")
            .args([tmp_script, "", "--unattended"])
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()?
    };

    // Fix ownership if running with sudo
    if std::env::var("SUDO_USER").is_ok() {
        let _ = Command::new("chown")
            .args(["-R", &format!("{}:{}",
                std::env::var("SUDO_USER").unwrap_or_default(),
                std::env::var("SUDO_USER").unwrap_or_default()
            ), omz_dir.to_str().unwrap_or("")])
            .status();
    }

    let _ = std::fs::remove_file(tmp_script);

    if !status.success() {
        anyhow::bail!("Oh My Zsh 安装失败");
    }
    Ok(())
}
```

**Why this works better:**
- Avoids shell expansion issues with `$(curl ...)`
- Easier to debug if download fails
- Temp file is cleaned up after execution
- `sudo -u $SUDO_USER` ensures oh-my-zsh installs to real user's home, not `/root/`
- `chown -R` fixes ownership when files are created during sudo execution

## Gotchas

1. **`ListItem` lifetime**: `make_list_items` needs explicit lifetime annotation:
   ```rust
   fn make_list_items<'a>(items: &'a [(String, String)], ...) -> Vec<ListItem<'a>>
   ```

2. **Closure lifetime in `Action::Execute`**: When returning a closure via `Box<dyn FnOnce>`, captured variables need `move`. **CRITICAL**: you CANNOT call `i18n::msg_success(lang, "name")` inside the closure because `lang` (even though `Copy`) won't live long enough for the `'static` closure. You MUST pre-compute the string into an owned `String` BEFORE the closure, then `move` it in:
   ```rust
   // WRONG — won't compile:
   Action::Execute(Box::new(|terminal| {
       run_in_terminal(terminal, || install())?;
       Ok(i18n::msg_success(lang, "Docker"))  // ERROR: lang doesn't live long enough
   }))

   // CORRECT — pre-compute, then move:
   let success_msg = i18n::msg_success(lang, "Docker");
   Action::Execute(Box::new(move |terminal| {
       run_in_terminal(terminal, || install())?;
       Ok(success_msg)
   }))
   ```

3. **Recursive enum**: If a Page variant contains a struct that references Page (e.g., "back" navigation), use `Box`:
   ```rust
   Status(Box<StatusData>)  // NOT Status(StatusData)
   ```

4. **`Block` lifetime**: `styled_block` return type needs `Block<'_>`:
   ```rust
   fn styled_block(title: &str) -> Block<'_> { ... }
   ```

5. **Always call `terminal.clear()` after re-entering alternate screen** to avoid rendering artifacts.

6. **Menu dispatch with closures for each action**: Instead of deeply nested match arms, use index-based dispatch:
   ```rust
   let idx = app.action_index;
   Action::Execute(Box::new(move |terminal| {
       run_in_terminal(terminal, || match idx {
           0 => install_docker(),
           1 => install_compose(),
           2 => add_user_to_group(),
           _ => Ok(()),
       })?;
       Ok(after_msg)
   }))
   ```
