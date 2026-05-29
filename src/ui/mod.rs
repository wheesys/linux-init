use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap},
    Frame, Terminal,
};
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent},
    execute, terminal,
};
use std::io::{self, Stdout};

use crate::app::*;

pub type Term = Terminal<CrosstermBackend<Stdout>>;

// ── Colors ──────────────────────────────────────────────────
const C_PRIMARY: Color = Color::Cyan;
const C_SUCCESS: Color = Color::Green;
const C_WARN: Color = Color::Yellow;
const C_ERROR: Color = Color::Red;
const C_DIM: Color = Color::DarkGray;
const C_HIGHLIGHT_BG: Color = Color::DarkGray;

// ── Terminal helpers ────────────────────────────────────────
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

fn run_in_terminal<F: FnOnce() -> anyhow::Result<()>>(
    terminal: &mut Term,
    f: F,
) -> anyhow::Result<()> {
    terminal::disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        crossterm::cursor::Show,
        terminal::LeaveAlternateScreen
    )?;

    let result = f();

    terminal::enable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        crossterm::cursor::Hide,
        terminal::EnterAlternateScreen
    )?;
    terminal.clear()?;

    result
}

// ── Action type ─────────────────────────────────────────────
pub enum Action {
    Quit,
    Execute(Box<dyn FnOnce(&mut Term) -> anyhow::Result<String>>),
}

// ── Render dispatcher ───────────────────────────────────────
pub fn render(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(3), Constraint::Length(1)])
        .split(frame.area());

    match &app.page {
        Page::MainMenu => render_main_menu(frame, app, chunks[0]),
        Page::Shell => render_shell(frame, app, chunks[0]),
        Page::ShellZshTheme => render_theme(frame, app, chunks[0]),
        Page::ShellZshPlugins => render_plugins(frame, app, chunks[0]),
        Page::Docker => render_docker(frame, app, chunks[0]),
        Page::Ssh => render_ssh(frame, app, chunks[0]),
        Page::Tools => render_tools(frame, app, chunks[0]),
        Page::Locale => render_locale(frame, app, chunks[0]),
        Page::Status(data) => render_status(frame, app, chunks[0], data),
    }

    render_status_bar(frame, app, chunks[1]);
}

fn render_status_bar(frame: &mut Frame, app: &App, area: Rect) {
    let keys = match &app.page {
        Page::MainMenu => "↑↓ 导航  Enter 选择  q 退出",
        Page::Status(_) => "↑↓ 滚动  Esc 返回",
        Page::Tools => "↑↓ 导航  Space 切换  a 全选  Enter 安装  Esc 返回",
        Page::ShellZshPlugins => "↑↓ 导航  Space 切换  Enter 确认  Esc 返回",
        _ => "↑↓ 导航  Enter 选择  Esc 返回  q 退出",
    };
    let text = Line::from(vec![
        Span::styled(
            format!(" {} ", app.distro),
            Style::default().fg(C_PRIMARY).add_modifier(Modifier::BOLD),
        ),
        Span::styled(" │ ", Style::default().fg(C_DIM)),
        Span::styled(keys, Style::default().fg(C_DIM)),
    ]);
    let bar = Paragraph::new(text)
        .style(Style::default().bg(Color::Rgb(30, 30, 30)));
    frame.render_widget(bar, area);
}

// ── Helper: centered rect ──────────────────────────────────
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup[1])[1]
}

fn styled_block(title: &str) -> Block<'_> {
    Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(C_PRIMARY))
        .title(Span::styled(
            format!(" {} ", title),
            Style::default()
                .fg(C_PRIMARY)
                .add_modifier(Modifier::BOLD),
        ))
}

fn make_list_items<'a>(items: &'a [(String, String)], selected: usize, statuses: &[bool]) -> Vec<ListItem<'a>> {
    items
        .iter()
        .enumerate()
        .map(|(i, (name, desc))| {
            let status = if statuses.get(i).copied().unwrap_or(false) {
                Span::styled("✓ ", Style::default().fg(C_SUCCESS))
            } else {
                Span::styled("  ", Style::default().fg(C_DIM))
            };
            let marker = if i == selected {
                Span::styled("▸ ", Style::default().fg(C_PRIMARY).add_modifier(Modifier::BOLD))
            } else {
                Span::raw("  ")
            };
            let name_span = Span::styled(
                name.as_str(),
                if i == selected {
                    Style::default().fg(Color::White).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::White)
                },
            );
            let desc_span = Span::styled(
                format!("  {}", desc),
                Style::default().fg(C_DIM),
            );
            ListItem::new(Line::from(vec![marker, status, name_span, desc_span]))
        })
        .collect()
}

// ── Page: Main Menu ─────────────────────────────────────────
fn render_main_menu(frame: &mut Frame, app: &App, area: Rect) {
    let block = styled_block("Linux Init — 环境初始化向导");

    let items: Vec<ListItem> = MAIN_MENU_ITEMS
        .iter()
        .enumerate()
        .map(|(i, (name, desc))| {
            let marker = if i == app.menu_index {
                Span::styled("  ▸ ", Style::default().fg(C_PRIMARY).add_modifier(Modifier::BOLD))
            } else {
                Span::raw("    ")
            };
            let name_s = Span::styled(
                *name,
                if i == app.menu_index {
                    Style::default().fg(Color::White).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::White)
                },
            );
            let desc_s = Span::styled(
                format!("  — {}", desc),
                Style::default().fg(C_DIM),
            );
            ListItem::new(Line::from(vec![marker, name_s, desc_s]))
        })
        .collect();

    let list = List::new(items)
        .block(block)
        .highlight_style(
            Style::default()
                .bg(C_HIGHLIGHT_BG)
                .add_modifier(Modifier::BOLD),
        );

    let mut state = ListState::default().with_selected(Some(app.menu_index));
    frame.render_stateful_widget(list, area, &mut state);
}

// ── Page: Shell ─────────────────────────────────────────────
fn render_shell(frame: &mut Frame, app: &App, area: Rect) {
    let block = styled_block("Shell 配置");

    let mut all_items: Vec<(String, String)> = vec![];
    let mut statuses: Vec<bool> = vec![];

    // Item 0: Install zsh
    all_items.push(("安装 Zsh".into(), "现代交互式 shell".into()));
    statuses.push(app.zsh_installed);

    // Item 1: Install Oh My Zsh
    all_items.push(("安装 Oh My Zsh".into(), "zsh 配置管理框架".into()));
    statuses.push(app.omz_installed);

    if app.omz_installed || app.omz_configured {
        // Item 2: Theme
        all_items.push(("选择主题".into(), format!("当前: {}", app.selected_theme)));
        statuses.push(false);

        // Item 3: Plugins
        let plugin_count = app.selected_plugins.len();
        all_items.push(("选择插件".into(), format!("已选 {} 个", plugin_count)));
        statuses.push(false);
    }

    // Item: Set default shell
    if app.zsh_installed {
        all_items.push(("设为默认 Shell".into(), "将 zsh 设为登录 shell".into()));
        statuses.push(app.default_shell_set);
    }

    let items = make_list_items(&all_items, app.shell_index, &statuses);

    let list = List::new(items)
        .block(block)
        .highlight_style(Style::default().bg(C_HIGHLIGHT_BG).add_modifier(Modifier::BOLD));

    let mut state = ListState::default().with_selected(Some(app.shell_index));
    frame.render_stateful_widget(list, area, &mut state);
}

// ── Page: Theme selection ───────────────────────────────────
fn render_theme(frame: &mut Frame, app: &App, area: Rect) {
    let block = styled_block("Oh My Zsh — 选择主题");

    let items: Vec<ListItem> = THEMES
        .iter()
        .enumerate()
        .map(|(i, (name, desc))| {
            let is_current = *name == app.selected_theme;
            let marker = if i == app.shell_theme_index {
                Span::styled("  ▸ ", Style::default().fg(C_PRIMARY).add_modifier(Modifier::BOLD))
            } else {
                Span::raw("    ")
            };
            let check = if is_current {
                Span::styled("● ", Style::default().fg(C_SUCCESS))
            } else {
                Span::styled("○ ", Style::default().fg(C_DIM))
            };
            let name_s = Span::styled(
                *name,
                if i == app.shell_theme_index {
                    Style::default().fg(Color::White).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::White)
                },
            );
            let desc_s = Span::styled(format!("  {}", desc), Style::default().fg(C_DIM));
            ListItem::new(Line::from(vec![marker, check, name_s, desc_s]))
        })
        .collect();

    let list = List::new(items)
        .block(block)
        .highlight_style(Style::default().bg(C_HIGHLIGHT_BG).add_modifier(Modifier::BOLD));

    let mut state = ListState::default().with_selected(Some(app.shell_theme_index));
    frame.render_stateful_widget(list, area, &mut state);
}

// ── Page: Plugin selection ──────────────────────────────────
fn render_plugins(frame: &mut Frame, app: &App, area: Rect) {
    let block = styled_block("Oh My Zsh — 选择插件 (Space 切换)");

    let items: Vec<ListItem> = PLUGINS
        .iter()
        .enumerate()
        .map(|(i, (name, kind, desc))| {
            let is_selected = app.selected_plugins.contains(&name.to_string());
            let marker = if i == app.plugin_index {
                Span::styled("  ▸ ", Style::default().fg(C_PRIMARY).add_modifier(Modifier::BOLD))
            } else {
                Span::raw("    ")
            };
            let check = if is_selected {
                Span::styled("■ ", Style::default().fg(C_SUCCESS))
            } else {
                Span::styled("□ ", Style::default().fg(C_DIM))
            };
            let name_s = Span::styled(
                *name,
                if i == app.plugin_index {
                    Style::default().fg(Color::White).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::White)
                },
            );
            let kind_s = Span::styled(
                format!(" [{}]", kind),
                Style::default().fg(C_WARN),
            );
            let desc_s = Span::styled(format!(" {}", desc), Style::default().fg(C_DIM));
            ListItem::new(Line::from(vec![marker, check, name_s, kind_s, desc_s]))
        })
        .collect();

    let list = List::new(items)
        .block(block)
        .highlight_style(Style::default().bg(C_HIGHLIGHT_BG).add_modifier(Modifier::BOLD));

    let mut state = ListState::default().with_selected(Some(app.plugin_index));
    frame.render_stateful_widget(list, area, &mut state);
}

// ── Page: Docker ────────────────────────────────────────────
fn render_docker(frame: &mut Frame, app: &App, area: Rect) {
    let block = styled_block("Docker 安装与配置");

    let items: Vec<(String, String)> = DOCKER_MENU_ITEMS
        .iter()
        .map(|(n, d)| (n.to_string(), d.to_string()))
        .collect();
    let statuses = vec![
        app.docker_installed,
        app.compose_installed,
        app.docker_user_configured,
        false,
    ];
    let items = make_list_items(&items, app.docker_index, &statuses);

    let list = List::new(items)
        .block(block)
        .highlight_style(Style::default().bg(C_HIGHLIGHT_BG).add_modifier(Modifier::BOLD));

    let mut state = ListState::default().with_selected(Some(app.docker_index));
    frame.render_stateful_widget(list, area, &mut state);
}

// ── Page: SSH ───────────────────────────────────────────────
fn render_ssh(frame: &mut Frame, app: &App, area: Rect) {
    let block = styled_block("SSH Key 生成");

    let items: Vec<(String, String)> = SSH_MENU_ITEMS
        .iter()
        .map(|(n, d)| (n.to_string(), d.to_string()))
        .collect();
    let statuses = vec![app.ssh_key_exists, app.ssh_key_exists, false];
    let items = make_list_items(&items, app.ssh_index, &statuses);

    let list = List::new(items)
        .block(block)
        .highlight_style(Style::default().bg(C_HIGHLIGHT_BG).add_modifier(Modifier::BOLD));

    let mut state = ListState::default().with_selected(Some(app.ssh_index));
    frame.render_stateful_widget(list, area, &mut state);

    if !app.last_pubkey.is_empty() {
        let popup = centered_rect(80, 40, area);
        frame.render_widget(Clear, popup);
        let key_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(C_SUCCESS))
            .title(Span::styled(" 公钥 (Esc 关闭) ", Style::default().fg(C_SUCCESS).add_modifier(Modifier::BOLD)));
        let key_text = Paragraph::new(app.last_pubkey.as_str())
            .block(key_block)
            .wrap(Wrap { trim: false })
            .style(Style::default().fg(Color::White));
        frame.render_widget(key_text, popup);
    }
}

// ── Page: Tools ─────────────────────────────────────────────
fn render_tools(frame: &mut Frame, app: &App, area: Rect) {
    let block = styled_block("基础工具安装 (Space 切换, a 全选)");

    let items: Vec<ListItem> = TOOLS
        .iter()
        .enumerate()
        .map(|(i, (name, category, desc))| {
            let is_selected = app.selected_tools.get(i).copied().unwrap_or(false);
            let installed = crate::modules::tools::get_tool_status(name);
            let marker = if i == app.tool_index {
                Span::styled("  ▸ ", Style::default().fg(C_PRIMARY).add_modifier(Modifier::BOLD))
            } else {
                Span::raw("    ")
            };
            let check = if installed {
                Span::styled("✓ ", Style::default().fg(C_SUCCESS))
            } else if is_selected {
                Span::styled("■ ", Style::default().fg(C_WARN))
            } else {
                Span::styled("□ ", Style::default().fg(C_DIM))
            };
            let name_s = Span::styled(
                *name,
                if i == app.tool_index {
                    Style::default().fg(Color::White).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::White)
                },
            );
            let cat_s = Span::styled(format!(" [{}]", category), Style::default().fg(C_WARN));
            let desc_s = Span::styled(format!(" {}", desc), Style::default().fg(C_DIM));
            ListItem::new(Line::from(vec![marker, check, name_s, cat_s, desc_s]))
        })
        .collect();

    let list = List::new(items)
        .block(block)
        .highlight_style(Style::default().bg(C_HIGHLIGHT_BG).add_modifier(Modifier::BOLD));

    let mut state = ListState::default().with_selected(Some(app.tool_index));
    frame.render_stateful_widget(list, area, &mut state);
}

// ── Page: Locale ────────────────────────────────────────────
fn render_locale(frame: &mut Frame, app: &App, area: Rect) {
    let block = styled_block("中文环境配置");

    let items: Vec<(String, String)> = LOCALE_MENU_ITEMS
        .iter()
        .map(|(n, d)| (n.to_string(), d.to_string()))
        .collect();
    let statuses = vec![app.locale_configured, app.fonts_installed, app.fcitx_installed];
    let items = make_list_items(&items, app.locale_index, &statuses);

    let list = List::new(items)
        .block(block)
        .highlight_style(Style::default().bg(C_HIGHLIGHT_BG).add_modifier(Modifier::BOLD));

    let mut state = ListState::default().with_selected(Some(app.locale_index));
    frame.render_stateful_widget(list, area, &mut state);
}

// ── Page: Status ────────────────────────────────────────────
fn render_status(frame: &mut Frame, _app: &App, area: Rect, data: &StatusData) {
    let block = styled_block(&data.title);
    let inner = block.inner(area);

    let lines: Vec<Line> = data
        .lines
        .iter()
        .map(|l| {
            let style = if l.starts_with('✅') {
                Style::default().fg(C_SUCCESS)
            } else if l.starts_with('❌') {
                Style::default().fg(C_ERROR)
            } else if l.starts_with("执行:") {
                Style::default().fg(C_WARN)
            } else {
                Style::default().fg(Color::White)
            };
            Line::styled(l.as_str(), style)
        })
        .collect();

    let total = lines.len();
    let visible = inner.height.saturating_sub(1) as usize;
    let offset = total.saturating_sub(visible);

    let para = Paragraph::new(lines)
        .block(block)
        .scroll((offset as u16, 0))
        .wrap(Wrap { trim: false });

    frame.render_widget(para, area);
}

// ── Event handling ──────────────────────────────────────────
pub fn handle_event(terminal: &mut Term, app: &mut App) -> anyhow::Result<Option<Action>> {
    if !event::poll(std::time::Duration::from_millis(50))? {
        return Ok(None);
    }
    let ev = event::read()?;
    if let Event::Key(key) = ev {
        if key.code == KeyCode::Char('q') && !matches!(app.page, Page::ShellZshPlugins | Page::Tools | Page::Status(_)) {
            return Ok(Some(Action::Quit));
        }
        return handle_key(terminal, app, key);
    }
    Ok(None)
}

fn handle_key(terminal: &mut Term, app: &mut App, key: KeyEvent) -> anyhow::Result<Option<Action>> {
    match &app.page {
        Page::MainMenu => handle_main_menu(app, key),
        Page::Shell => handle_shell(terminal, app, key),
        Page::ShellZshTheme => handle_theme(app, key),
        Page::ShellZshPlugins => handle_plugins(app, key),
        Page::Docker => handle_docker(app, key),
        Page::Ssh => handle_ssh(app, key),
        Page::Tools => handle_tools(app, key),
        Page::Locale => handle_locale(app, key),
        Page::Status(_) => handle_status(app, key),
    }
}

// ── Key handlers per page ───────────────────────────────────

fn handle_main_menu(app: &mut App, key: KeyEvent) -> anyhow::Result<Option<Action>> {
    let max = MAIN_MENU_ITEMS.len();
    match key.code {
        KeyCode::Up => app.menu_index = app.menu_index.saturating_sub(1),
        KeyCode::Down => app.menu_index = (app.menu_index + 1).min(max - 1),
        KeyCode::Enter => {
            app.page = match app.menu_index {
                0 => Page::Shell,
                1 => Page::Docker,
                2 => Page::Ssh,
                3 => Page::Tools,
                4 => Page::Locale,
                _ => return Ok(None),
            };
            app.status_msg = "按 Esc 返回主菜单".into();
        }
        _ => {}
    }
    Ok(None)
}

fn handle_shell(terminal: &mut Term, app: &mut App, key: KeyEvent) -> anyhow::Result<Option<Action>> {
    let max = shell_menu_count(app);
    match key.code {
        KeyCode::Esc | KeyCode::Backspace => {
            app.page = Page::MainMenu;
            app.shell_index = 0;
        }
        KeyCode::Up => app.shell_index = app.shell_index.saturating_sub(1),
        KeyCode::Down => app.shell_index = (app.shell_index + 1).min(max.saturating_sub(1)),
        KeyCode::Enter => {
            return handle_shell_enter(terminal, app);
        }
        _ => {}
    }
    Ok(None)
}

fn shell_menu_count(app: &App) -> usize {
    let mut count = 2; // install zsh + install omz
    if app.omz_installed || app.omz_configured {
        count += 2; // theme + plugins
    }
    if app.zsh_installed {
        count += 1; // default shell
    }
    count
}

fn handle_shell_enter(terminal: &mut Term, app: &mut App) -> anyhow::Result<Option<Action>> {
    let mut idx = 0;

    // Item 0: Install zsh
    if app.shell_index == idx {
        if !app.zsh_installed {
            app.status_msg = "正在安装 zsh...".into();
            let result = run_in_terminal(terminal, || {
                crate::modules::shell::install_zsh()
            });
            match result {
                Ok(()) => {
                    app.zsh_installed = true;
                    app.status_msg = "✅ zsh 安装成功".into();
                }
                Err(e) => app.status_msg = format!("❌ zsh 安装失败: {}", e),
            }
        }
        return Ok(None);
    }
    idx += 1;

    // Item 1: Install Oh My Zsh
    if app.shell_index == idx {
        if !app.omz_installed {
            if !app.zsh_installed {
                app.status_msg = "❌ 请先安装 zsh".into();
                return Ok(None);
            }
            app.status_msg = "正在安装 Oh My Zsh...".into();
            let result = run_in_terminal(terminal, || {
                crate::modules::shell::install_oh_my_zsh()
            });
            match result {
                Ok(()) => {
                    app.omz_installed = true;
                    app.omz_configured = true;
                    app.status_msg = "✅ Oh My Zsh 安装成功".into();
                }
                Err(e) => app.status_msg = format!("❌ Oh My Zsh 安装失败: {}", e),
            }
        }
        return Ok(None);
    }
    idx += 1;

    // Theme / Plugins / Default shell (only if omz installed)
    if app.omz_installed || app.omz_configured {
        if app.shell_index == idx {
            app.page = Page::ShellZshTheme;
            app.shell_theme_index = THEMES
                .iter()
                .position(|(n, _)| *n == app.selected_theme)
                .unwrap_or(0);
            return Ok(None);
        }
        idx += 1;

        if app.shell_index == idx {
            app.page = Page::ShellZshPlugins;
            app.plugin_index = 0;
            return Ok(None);
        }
        idx += 1;
    }

    // Default shell
    if app.zsh_installed && app.shell_index == idx {
        if !app.default_shell_set {
            app.status_msg = "正在设置默认 Shell...".into();
            let result = run_in_terminal(terminal, || {
                crate::modules::shell::set_default_shell()
            });
            match result {
                Ok(()) => {
                    app.default_shell_set = true;
                    app.status_msg = "✅ zsh 已设为默认 Shell (重新登录生效)".into();
                }
                Err(e) => app.status_msg = format!("❌ 设置失败: {}", e),
            }
        }
    }

    Ok(None)
}

fn handle_theme(app: &mut App, key: KeyEvent) -> anyhow::Result<Option<Action>> {
    match key.code {
        KeyCode::Esc | KeyCode::Backspace => {
            app.page = Page::Shell;
        }
        KeyCode::Up => app.shell_theme_index = app.shell_theme_index.saturating_sub(1),
        KeyCode::Down => app.shell_theme_index = (app.shell_theme_index + 1).min(THEMES.len() - 1),
        KeyCode::Enter => {
            let theme = THEMES[app.shell_theme_index].0;
            app.selected_theme = theme.to_string();
            match crate::modules::shell::set_theme(theme) {
                Ok(()) => app.status_msg = format!("✅ 主题已设置为: {}", theme),
                Err(e) => app.status_msg = format!("❌ 主题设置失败: {}", e),
            }
            app.page = Page::Shell;
        }
        _ => {}
    }
    Ok(None)
}

fn handle_plugins(app: &mut App, key: KeyEvent) -> anyhow::Result<Option<Action>> {
    match key.code {
        KeyCode::Esc | KeyCode::Backspace => {
            // Save plugins on exit
            if !app.selected_plugins.is_empty() {
                match crate::modules::shell::set_plugins(&app.selected_plugins) {
                    Ok(()) => app.status_msg = format!("✅ 已配置 {} 个插件", app.selected_plugins.len()),
                    Err(e) => app.status_msg = format!("❌ 插件配置失败: {}", e),
                }
            }
            app.page = Page::Shell;
        }
        KeyCode::Up => app.plugin_index = app.plugin_index.saturating_sub(1),
        KeyCode::Down => app.plugin_index = (app.plugin_index + 1).min(PLUGINS.len() - 1),
        KeyCode::Char(' ') => {
            let name = PLUGINS[app.plugin_index].0.to_string();
            if let Some(pos) = app.selected_plugins.iter().position(|p| *p == name) {
                app.selected_plugins.remove(pos);
            } else {
                app.selected_plugins.push(name);
            }
            app.plugin_index = (app.plugin_index + 1).min(PLUGINS.len() - 1);
        }
        KeyCode::Enter => {
            // Save and go back
            if !app.selected_plugins.is_empty() {
                match crate::modules::shell::set_plugins(&app.selected_plugins) {
                    Ok(()) => app.status_msg = format!("✅ 已配置 {} 个插件", app.selected_plugins.len()),
                    Err(e) => app.status_msg = format!("❌ 插件配置失败: {}", e),
                }
            }
            app.page = Page::Shell;
        }
        _ => {}
    }
    Ok(None)
}

fn handle_docker(app: &mut App, key: KeyEvent) -> anyhow::Result<Option<Action>> {
    match key.code {
        KeyCode::Esc | KeyCode::Backspace => {
            app.page = Page::MainMenu;
            app.docker_index = 0;
        }
        KeyCode::Up => app.docker_index = app.docker_index.saturating_sub(1),
        KeyCode::Down => app.docker_index = (app.docker_index + 1).min(DOCKER_MENU_ITEMS.len() - 1),
        KeyCode::Enter => {
            match app.docker_index {
                0 => {
                    if !app.docker_installed {
                        app.status_msg = "正在安装 Docker...".into();
                        return Ok(Some(Action::Execute(Box::new(|terminal| {
                            run_in_terminal(terminal, || crate::modules::docker::install_docker())?;
                            Ok("✅ Docker 安装成功".into())
                        }))));
                    }
                }
                1 => {
                    if !app.compose_installed {
                        app.status_msg = "正在安装 Docker Compose...".into();
                        return Ok(Some(Action::Execute(Box::new(|terminal| {
                            run_in_terminal(terminal, || crate::modules::docker::install_compose())?;
                            Ok("✅ Docker Compose 安装成功".into())
                        }))));
                    }
                }
                2 => {
                    if !app.docker_user_configured {
                        app.status_msg = "正在配置 docker 用户组...".into();
                        return Ok(Some(Action::Execute(Box::new(|terminal| {
                            run_in_terminal(terminal, || crate::modules::docker::add_user_to_docker_group())?;
                            Ok("✅ 已将当前用户加入 docker 组 (重新登录生效)".into())
                        }))));
                    }
                }
                3 => {
                    app.status_msg = "正在启动 Docker 服务...".into();
                    return Ok(Some(Action::Execute(Box::new(|terminal| {
                        run_in_terminal(terminal, || crate::modules::docker::start_docker_service())?;
                        Ok("✅ Docker 服务已启动并设为开机自启".into())
                    }))));
                }
                _ => {}
            }
        }
        _ => {}
    }
    Ok(None)
}

fn handle_ssh(app: &mut App, key: KeyEvent) -> anyhow::Result<Option<Action>> {
    match key.code {
        KeyCode::Esc | KeyCode::Backspace => {
            app.last_pubkey.clear();
            app.page = Page::MainMenu;
            app.ssh_index = 0;
        }
        KeyCode::Up => app.ssh_index = app.ssh_index.saturating_sub(1),
        KeyCode::Down => app.ssh_index = (app.ssh_index + 1).min(SSH_MENU_ITEMS.len() - 1),
        KeyCode::Enter => {
            match app.ssh_index {
                0 => {
                    let email = std::env::var("USER").unwrap_or_else(|_| "user".into());
                    let email = format!("{}@localhost", email);
                    match crate::modules::ssh::generate_ed25519(&email) {
                        Ok(pubkey) => {
                            app.ssh_key_exists = true;
                            app.last_pubkey = pubkey;
                            app.status_msg = "✅ Ed25519 密钥已生成".into();
                        }
                        Err(e) => app.status_msg = format!("❌ 生成失败: {}", e),
                    }
                }
                1 => {
                    let email = std::env::var("USER").unwrap_or_else(|_| "user".into());
                    let email = format!("{}@localhost", email);
                    match crate::modules::ssh::generate_rsa(&email) {
                        Ok(pubkey) => {
                            app.ssh_key_exists = true;
                            app.last_pubkey = pubkey;
                            app.status_msg = "✅ RSA 4096 密钥已生成".into();
                        }
                        Err(e) => app.status_msg = format!("❌ 生成失败: {}", e),
                    }
                }
                2 => match crate::modules::ssh::read_public_key() {
                    Ok(pubkey) => {
                        app.last_pubkey = pubkey;
                    }
                    Err(e) => app.status_msg = format!("❌ 读取失败: {}", e),
                },
                _ => {}
            }
        }
        _ => {}
    }
    Ok(None)
}

fn handle_tools(app: &mut App, key: KeyEvent) -> anyhow::Result<Option<Action>> {
    match key.code {
        KeyCode::Esc | KeyCode::Backspace => {
            app.page = Page::MainMenu;
            app.tool_index = 0;
        }
        KeyCode::Up => app.tool_index = app.tool_index.saturating_sub(1),
        KeyCode::Down => app.tool_index = (app.tool_index + 1).min(TOOLS.len() - 1),
        KeyCode::Char(' ') => {
            let idx = app.tool_index;
            if let Some(val) = app.selected_tools.get_mut(idx) {
                *val = !*val;
            }
            app.tool_index = (app.tool_index + 1).min(TOOLS.len() - 1);
        }
        KeyCode::Char('a') => {
            let all_selected = app.selected_tools.iter().all(|v| *v);
            app.selected_tools.iter_mut().for_each(|v| *v = !all_selected);
        }
        KeyCode::Enter => {
            let selected: Vec<&str> = TOOLS
                .iter()
                .enumerate()
                .filter(|(i, _)| app.selected_tools.get(*i).copied().unwrap_or(false))
                .map(|(_, (name, _, _))| *name)
                .collect();

            if selected.is_empty() {
                app.status_msg = "请先选择要安装的工具".into();
                return Ok(None);
            }

            let count = selected.len();
            app.status_msg = format!("正在安装 {} 个工具...", count);
            let selected_owned: Vec<String> = selected.iter().map(|s| s.to_string()).collect();
            return Ok(Some(Action::Execute(Box::new(move |terminal| {
                run_in_terminal(terminal, || {
                    let refs: Vec<&str> = selected_owned.iter().map(|s| s.as_str()).collect();
                    crate::modules::tools::install_tools(&refs)
                })?;
                Ok(format!("✅ {} 个工具安装成功", count))
            }))));
        }
        _ => {}
    }
    Ok(None)
}

fn handle_locale(app: &mut App, key: KeyEvent) -> anyhow::Result<Option<Action>> {
    match key.code {
        KeyCode::Esc | KeyCode::Backspace => {
            app.page = Page::MainMenu;
            app.locale_index = 0;
        }
        KeyCode::Up => app.locale_index = app.locale_index.saturating_sub(1),
        KeyCode::Down => {
            app.locale_index = (app.locale_index + 1).min(LOCALE_MENU_ITEMS.len() - 1)
        }
        KeyCode::Enter => match app.locale_index {
            0 => {
                app.status_msg = "正在配置中文 locale...".into();
                return Ok(Some(Action::Execute(Box::new(|terminal| {
                    run_in_terminal(terminal, || crate::modules::locale::configure_locale())?;
                    Ok("✅ 中文 locale 配置成功".into())
                }))));
            }
            1 => {
                app.status_msg = "正在安装中文字体...".into();
                return Ok(Some(Action::Execute(Box::new(|terminal| {
                    run_in_terminal(terminal, || crate::modules::locale::install_cjk_fonts())?;
                    Ok("✅ 中文字体安装成功".into())
                }))));
            }
            2 => {
                app.status_msg = "正在安装 Fcitx5 输入法...".into();
                return Ok(Some(Action::Execute(Box::new(|terminal| {
                    run_in_terminal(terminal, || crate::modules::locale::install_fcitx5())?;
                    Ok("✅ Fcitx5 输入法安装成功 (重新登录生效)".into())
                }))));
            }
            _ => {}
        },
        _ => {}
    }
    Ok(None)
}

fn handle_status(app: &mut App, key: KeyEvent) -> anyhow::Result<Option<Action>> {
    match key.code {
        KeyCode::Esc | KeyCode::Backspace | KeyCode::Enter => {
            if let Page::Status(data) = &app.page {
                app.page = data.back.clone();
            }
        }
        _ => {}
    }
    Ok(None)
}
