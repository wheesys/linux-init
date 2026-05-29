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

## Gotchas

1. **`ListItem` lifetime**: `make_list_items` needs explicit lifetime annotation:
   ```rust
   fn make_list_items<'a>(items: &'a [(String, String)], ...) -> Vec<ListItem<'a>>
   ```

2. **Closure lifetime in `Action::Execute`**: When returning a closure via `Box<dyn FnOnce>`, captured variables need `move`:
   ```rust
   let owned_data = data.clone();
   Action::Execute(Box::new(move |terminal| {
       // use owned_data here
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
