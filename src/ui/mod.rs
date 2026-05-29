use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap},
    Frame, Terminal,
};
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute, terminal,
};
use std::io::{self, Stdout};

use crate::app::*;
use crate::i18n::{self, Lang};

pub type Term = Terminal<CrosstermBackend<Stdout>>;

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
        Page::LangSelect => render_lang_select(frame, app, chunks[0]),
        Page::MainMenu => render_main_menu(frame, app, chunks[0]),
        Page::Shell => render_shell(frame, app, chunks[0]),
        Page::ShellZshTheme => render_theme(frame, app, chunks[0]),
        Page::ShellZshPlugins => render_plugins(frame, app, chunks[0]),
        Page::Docker => render_docker(frame, app, chunks[0]),
        Page::Ssh => render_ssh(frame, app, chunks[0]),
        Page::SshServer => render_ssh_server(frame, app, chunks[0]),
        Page::Tools => render_tools(frame, app, chunks[0]),
        Page::Vim => render_vim(frame, app, chunks[0]),
        Page::VimPlugins => render_vim_plugins(frame, app, chunks[0]),
        Page::Nvm => render_nvm(frame, app, chunks[0]),
        Page::Locale => render_locale(frame, app, chunks[0]),
        Page::Status(data) => render_status(frame, app, chunks[0], data),
    }

    render_status_bar(frame, app, chunks[1]);
}

fn render_status_bar(frame: &mut Frame, app: &App, area: Rect) {
    let lang = app.lang;
    let keys = match &app.page {
        Page::LangSelect => "↑↓ nav  Enter select",
        Page::MainMenu => i18n::statusbar_main(lang),
        Page::Status(_) => i18n::statusbar_status(lang),
        Page::Tools | Page::VimPlugins => i18n::statusbar_tools(lang),
        Page::ShellZshPlugins => i18n::statusbar_multi(lang),
        _ => i18n::statusbar_nav(lang),
    };
    let text = Line::from(vec![
        Span::styled(
            format!(" {} | {} ", app.distro, lang),
            Style::default().fg(C_PRIMARY).add_modifier(Modifier::BOLD),
        ),
        Span::styled(" │ ", Style::default().fg(C_DIM)),
        Span::styled(keys, Style::default().fg(C_DIM)),
    ]);
    let bar = Paragraph::new(text).style(Style::default().bg(Color::Rgb(30, 30, 30)));
    frame.render_widget(bar, area);
}

fn styled_block(title: &str) -> Block<'_> {
    Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(C_PRIMARY))
        .title(Span::styled(
            format!(" {} ", title),
            Style::default().fg(C_PRIMARY).add_modifier(Modifier::BOLD),
        ))
}

fn make_list_items<'a>(
    items: &'a [(String, String)],
    selected: usize,
    statuses: &[bool],
) -> Vec<ListItem<'a>> {
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
                    Style::default().fg(Color::Reset).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::Reset)
                },
            );
            let desc_span = Span::styled(format!("  {}", desc), Style::default().fg(C_DIM));
            ListItem::new(Line::from(vec![marker, status, name_span, desc_span]))
        })
        .collect()
}

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

// ── Page: Language Select ───────────────────────────────────
fn render_lang_select(frame: &mut Frame, app: &App, area: Rect) {
    let block = styled_block(i18n::lang_select_title());
    let langs = [(Lang::Chinese, "中文 / Chinese"), (Lang::English, "English")];
    let items: Vec<ListItem> = langs
        .iter()
        .enumerate()
        .map(|(i, (_, label))| {
            let marker = if i == app.lang_index {
                Span::styled("  ▸ ", Style::default().fg(C_PRIMARY).add_modifier(Modifier::BOLD))
            } else {
                Span::raw("    ")
            };
            let is_current = (i == 0 && app.lang == Lang::Chinese)
                || (i == 1 && app.lang == Lang::English);
            let check = if is_current {
                Span::styled("● ", Style::default().fg(C_SUCCESS))
            } else {
                Span::styled("○ ", Style::default().fg(C_DIM))
            };
            let name_s = Span::styled(
                *label,
                if i == app.lang_index {
                    Style::default().fg(Color::Reset).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::Reset)
                },
            );
            ListItem::new(Line::from(vec![marker, check, name_s]))
        })
        .collect();

    let list = List::new(items)
        .block(block)
        .highlight_style(Style::default().bg(C_HIGHLIGHT_BG).add_modifier(Modifier::BOLD));

    let mut state = ListState::default().with_selected(Some(app.lang_index));
    frame.render_stateful_widget(list, area, &mut state);
}

// ── Page: Main Menu ─────────────────────────────────────────
fn render_main_menu(frame: &mut Frame, app: &App, area: Rect) {
    let lang = app.lang;
    let block = styled_block(i18n::main_title(lang));
    let menu = i18n::main_menu(lang);

    let items: Vec<ListItem> = menu
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
                    Style::default().fg(Color::Reset).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::Reset)
                },
            );
            let desc_s = Span::styled(format!("  — {}", desc), Style::default().fg(C_DIM));
            ListItem::new(Line::from(vec![marker, name_s, desc_s]))
        })
        .collect();

    let list = List::new(items)
        .block(block)
        .highlight_style(Style::default().bg(C_HIGHLIGHT_BG).add_modifier(Modifier::BOLD));

    let mut state = ListState::default().with_selected(Some(app.menu_index));
    frame.render_stateful_widget(list, area, &mut state);
}

// ── Page: Shell ─────────────────────────────────────────────
fn render_shell(frame: &mut Frame, app: &App, area: Rect) {
    let lang = app.lang;
    let block = styled_block(i18n::shell_title(lang));

    let mut all_items: Vec<(String, String)> = vec![];
    let mut statuses: Vec<bool> = vec![];

    let (n, d) = i18n::shell_install_zsh(lang);
    all_items.push((n.into(), d.into()));
    statuses.push(app.zsh_installed);

    let (n, d) = i18n::shell_install_omz(lang);
    all_items.push((n.into(), d.into()));
    statuses.push(app.omz_installed);

    if app.omz_installed || app.omz_configured {
        let (n, d) = i18n::shell_select_theme(lang, &app.selected_theme);
        all_items.push((n, d));
        statuses.push(false);

        let (n, d) = i18n::shell_select_plugins(lang, app.selected_plugins.len());
        all_items.push((n, d));
        statuses.push(false);
    }

    if app.zsh_installed {
        let (n, d) = i18n::shell_set_default(lang);
        all_items.push((n.into(), d.into()));
        statuses.push(app.default_shell_set);
    }

    let items = make_list_items(&all_items, app.shell_index, &statuses);
    let list = List::new(items)
        .block(block)
        .highlight_style(Style::default().bg(C_HIGHLIGHT_BG).add_modifier(Modifier::BOLD));

    let mut state = ListState::default().with_selected(Some(app.shell_index));
    frame.render_stateful_widget(list, area, &mut state);
}

// ── Page: Theme ─────────────────────────────────────────────
fn render_theme(frame: &mut Frame, app: &App, area: Rect) {
    let lang = app.lang;
    let block = styled_block(i18n::theme_title(lang));

    let items: Vec<ListItem> = THEMES
        .iter()
        .enumerate()
        .map(|(i, (name, _))| {
            let desc = i18n::theme_desc(lang, name);
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
                    Style::default().fg(Color::Reset).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::Reset)
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

// ── Page: OMZ Plugins ──────────────────────────────────────
fn render_plugins(frame: &mut Frame, app: &App, area: Rect) {
    let lang = app.lang;
    let block = styled_block(i18n::plugin_title(lang));

    let items: Vec<ListItem> = PLUGINS
        .iter()
        .enumerate()
        .map(|(i, (name, kind))| {
            let desc = i18n::plugin_desc(lang, name);
            let kind_label = i18n::plugin_kind(lang, kind);
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
                    Style::default().fg(Color::Reset).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::Reset)
                },
            );
            let kind_s = Span::styled(format!(" [{}]", kind_label), Style::default().fg(C_WARN));
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
    let lang = app.lang;
    let block = styled_block(i18n::docker_title(lang));
    let menu = i18n::docker_menu(lang);

    let items: Vec<(String, String)> = menu
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

// ── Page: SSH Key ──────────────────────────────────────────
fn render_ssh(frame: &mut Frame, app: &App, area: Rect) {
    let lang = app.lang;
    let block = styled_block(i18n::ssh_title(lang));
    let menu = i18n::ssh_menu(lang);

    let items: Vec<(String, String)> = menu
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
            .title(Span::styled(
                i18n::ssh_pubkey_title(lang),
                Style::default().fg(C_SUCCESS).add_modifier(Modifier::BOLD),
            ));
        let key_text = Paragraph::new(app.last_pubkey.as_str())
            .block(key_block)
            .wrap(Wrap { trim: false })
            .style(Style::default().fg(Color::Reset));
        frame.render_widget(key_text, popup);
    }
}

// ── Page: SSH Server ───────────────────────────────────────
fn render_ssh_server(frame: &mut Frame, app: &App, area: Rect) {
    let lang = app.lang;
    let block = styled_block(i18n::ssh_server_title(lang));
    let menu = i18n::ssh_server_menu(lang);

    let items: Vec<(String, String)> = menu
        .iter()
        .map(|(n, d)| (n.to_string(), d.to_string()))
        .collect();
    let statuses = vec![
        app.sshd_installed,
        app.sshd_root_disabled,
        false,
    ];
    let items = make_list_items(&items, app.ssh_server_index, &statuses);
    let list = List::new(items)
        .block(block)
        .highlight_style(Style::default().bg(C_HIGHLIGHT_BG).add_modifier(Modifier::BOLD));

    let mut state = ListState::default().with_selected(Some(app.ssh_server_index));
    frame.render_stateful_widget(list, area, &mut state);
}

// ── Page: Tools ─────────────────────────────────────────────
fn render_tools(frame: &mut Frame, app: &App, area: Rect) {
    let lang = app.lang;
    let block = styled_block(i18n::tools_title(lang));

    let items: Vec<ListItem> = TOOLS
        .iter()
        .enumerate()
        .map(|(i, (name, category, _))| {
            let desc = i18n::tool_desc(lang, name);
            let cat_label = i18n::tool_category(lang, category);
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
                    Style::default().fg(Color::Reset).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::Reset)
                },
            );
            let cat_s = Span::styled(format!(" [{}]", cat_label), Style::default().fg(C_WARN));
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

// ── Page: Vim ───────────────────────────────────────────────
fn render_vim(frame: &mut Frame, app: &App, area: Rect) {
    let lang = app.lang;
    let block = styled_block(i18n::vim_title(lang));
    let menu = i18n::vim_menu(lang);

    let statuses = vec![app.vim_installed, app.vundle_installed, false];
    let items = make_list_items(&menu, app.vim_index, &statuses);
    let list = List::new(items)
        .block(block)
        .highlight_style(Style::default().bg(C_HIGHLIGHT_BG).add_modifier(Modifier::BOLD));

    let mut state = ListState::default().with_selected(Some(app.vim_index));
    frame.render_stateful_widget(list, area, &mut state);
}

// ── Page: Vim Plugins ──────────────────────────────────────
fn render_vim_plugins(frame: &mut Frame, app: &App, area: Rect) {
    let lang = app.lang;
    let block = styled_block(i18n::vim_plugin_title(lang));
    let plugins = crate::modules::vim::VIM_PLUGINS;

    let items: Vec<ListItem> = plugins
        .iter()
        .enumerate()
        .map(|(i, (name, _repo))| {
            let desc = i18n::vim_plugin_desc(lang, name);
            let is_selected = app.selected_vim_plugins.contains(&i);
            let marker = if i == app.vim_plugin_index {
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
                if i == app.vim_plugin_index {
                    Style::default().fg(Color::Reset).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::Reset)
                },
            );
            let desc_s = Span::styled(format!(" {}", desc), Style::default().fg(C_DIM));
            ListItem::new(Line::from(vec![marker, check, name_s, desc_s]))
        })
        .collect();

    let list = List::new(items)
        .block(block)
        .highlight_style(Style::default().bg(C_HIGHLIGHT_BG).add_modifier(Modifier::BOLD));

    let mut state = ListState::default().with_selected(Some(app.vim_plugin_index));
    frame.render_stateful_widget(list, area, &mut state);
}

// ── Page: NVM ───────────────────────────────────────────────
fn render_nvm(frame: &mut Frame, app: &App, area: Rect) {
    let lang = app.lang;
    let block = styled_block(i18n::nvm_title(lang));
    let menu = i18n::nvm_menu(lang);

    let items: Vec<(String, String)> = menu
        .iter()
        .map(|(n, d)| (n.to_string(), d.to_string()))
        .collect();
    let statuses = vec![app.nvm_installed, app.node_installed, false];
    let items = make_list_items(&items, app.nvm_index, &statuses);
    let list = List::new(items)
        .block(block)
        .highlight_style(Style::default().bg(C_HIGHLIGHT_BG).add_modifier(Modifier::BOLD));

    let mut state = ListState::default().with_selected(Some(app.nvm_index));
    frame.render_stateful_widget(list, area, &mut state);
}

// ── Page: Locale ───────────────────────────────────────────
fn render_locale(frame: &mut Frame, app: &App, area: Rect) {
    let lang = app.lang;
    let block = styled_block(i18n::locale_title(lang));
    let menu = i18n::locale_menu(lang);

    let items: Vec<(String, String)> = menu
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

// ── Page: Status ───────────────────────────────────────────
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
            } else if l.starts_with("执行:") || l.starts_with("Run:") {
                Style::default().fg(C_WARN)
            } else {
                Style::default().fg(Color::Reset)
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
        // Ctrl+C 退出（终端通用方案）
        if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
            return Ok(Some(Action::Quit));
        }
        if key.code == KeyCode::Char('q')
            && !matches!(
                app.page,
                Page::LangSelect
                    | Page::ShellZshPlugins
                    | Page::Tools
                    | Page::VimPlugins
                    | Page::Status(_)
            )
        {
            return Ok(Some(Action::Quit));
        }
        return handle_key(terminal, app, key);
    }
    Ok(None)
}

fn handle_key(
    terminal: &mut Term,
    app: &mut App,
    key: KeyEvent,
) -> anyhow::Result<Option<Action>> {
    match &app.page {
        Page::LangSelect => handle_lang_select(app, key),
        Page::MainMenu => handle_main_menu(app, key),
        Page::Shell => handle_shell(terminal, app, key),
        Page::ShellZshTheme => handle_theme(app, key),
        Page::ShellZshPlugins => handle_plugins(app, key),
        Page::Docker => handle_docker(app, key),
        Page::Ssh => handle_ssh(app, key),
        Page::SshServer => handle_ssh_server(app, key),
        Page::Tools => handle_tools(app, key),
        Page::Vim => handle_vim(terminal, app, key),
        Page::VimPlugins => handle_vim_plugins(app, key),
        Page::Nvm => handle_nvm(terminal, app, key),
        Page::Locale => handle_locale(app, key),
        Page::Status(_) => handle_status(app, key),
    }
}

// ── Key: Language select ───────────────────────────────────
fn handle_lang_select(app: &mut App, key: KeyEvent) -> anyhow::Result<Option<Action>> {
    match key.code {
        KeyCode::Up => app.lang_index = app.lang_index.saturating_sub(1),
        KeyCode::Down => app.lang_index = (app.lang_index + 1).min(1),
        KeyCode::Enter => {
            app.lang = if app.lang_index == 0 {
                Lang::Chinese
            } else {
                Lang::English
            };
            app.lang_selected = true;
            app.page = Page::MainMenu;
            app.status_msg = i18n::msg_press_esc(app.lang).into();
        }
        _ => {}
    }
    Ok(None)
}

// ── Key: Main menu ─────────────────────────────────────────
fn handle_main_menu(app: &mut App, key: KeyEvent) -> anyhow::Result<Option<Action>> {
    let max = i18n::main_menu(app.lang).len();
    match key.code {
        KeyCode::Up => app.menu_index = app.menu_index.saturating_sub(1),
        KeyCode::Down => app.menu_index = (app.menu_index + 1).min(max - 1),
        KeyCode::Enter => {
            app.page = match app.menu_index {
                0 => Page::Shell,
                1 => Page::Docker,
                2 => Page::Ssh,
                3 => Page::Tools,
                4 => Page::SshServer,
                5 => Page::Vim,
                6 => Page::Nvm,
                7 => Page::Locale,
                _ => return Ok(None),
            };
            app.status_msg = i18n::msg_press_esc(app.lang).into();
        }
        _ => {}
    }
    Ok(None)
}

// ── Key: Shell ─────────────────────────────────────────────
fn handle_shell(
    terminal: &mut Term,
    app: &mut App,
    key: KeyEvent,
) -> anyhow::Result<Option<Action>> {
    let max = shell_menu_count(app);
    match key.code {
        KeyCode::Esc | KeyCode::Backspace => {
            app.page = Page::MainMenu;
            app.shell_index = 0;
        }
        KeyCode::Up => app.shell_index = app.shell_index.saturating_sub(1),
        KeyCode::Down => app.shell_index = (app.shell_index + 1).min(max.saturating_sub(1)),
        KeyCode::Enter => return handle_shell_enter(terminal, app),
        _ => {}
    }
    Ok(None)
}

fn shell_menu_count(app: &App) -> usize {
    let mut count = 2;
    if app.omz_installed || app.omz_configured {
        count += 2;
    }
    if app.zsh_installed {
        count += 1;
    }
    count
}

fn handle_shell_enter(
    terminal: &mut Term,
    app: &mut App,
) -> anyhow::Result<Option<Action>> {
    let lang = app.lang;
    let mut idx = 0;

    if app.shell_index == idx {
        if !app.zsh_installed {
            app.status_msg = i18n::msg_installing(lang, "zsh");
            let result =
                run_in_terminal(terminal, || crate::modules::shell::install_zsh());
            match result {
                Ok(()) => {
                    app.zsh_installed = true;
                    app.status_msg = i18n::msg_success(lang, "zsh");
                }
                Err(e) => app.status_msg = i18n::msg_fail(lang, "zsh", &e.to_string()),
            }
        }
        return Ok(None);
    }
    idx += 1;

    if app.shell_index == idx {
        if !app.omz_installed {
            if !app.zsh_installed {
                app.status_msg = match lang {
                    Lang::Chinese => "❌ 请先安装 zsh".into(),
                    Lang::English => "❌ Please install zsh first".into(),
                };
                return Ok(None);
            }
            app.status_msg = i18n::msg_installing(lang, "Oh My Zsh");
            let result =
                run_in_terminal(terminal, || crate::modules::shell::install_oh_my_zsh());
            match result {
                Ok(()) => {
                    app.omz_installed = true;
                    app.omz_configured = true;
                    app.status_msg = i18n::msg_success(lang, "Oh My Zsh");
                }
                Err(e) => {
                    app.status_msg = i18n::msg_fail(lang, "Oh My Zsh", &e.to_string())
                }
            }
        }
        return Ok(None);
    }
    idx += 1;

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

    if app.zsh_installed && app.shell_index == idx {
        if !app.default_shell_set {
            let msg = match lang {
                Lang::Chinese => "正在设置默认 Shell...",
                Lang::English => "Setting default shell...",
            };
            app.status_msg = msg.into();
            let result =
                run_in_terminal(terminal, || crate::modules::shell::set_default_shell());
            match result {
                Ok(()) => {
                    app.default_shell_set = true;
                    app.status_msg = match lang {
                        Lang::Chinese => "✅ zsh 已设为默认 Shell (重新登录生效)".into(),
                        Lang::English => "✅ zsh set as default shell (re-login required)".into(),
                    };
                }
                Err(e) => {
                    app.status_msg =
                        i18n::msg_fail(lang, "default shell", &e.to_string())
                }
            }
        }
    }

    Ok(None)
}

fn handle_theme(app: &mut App, key: KeyEvent) -> anyhow::Result<Option<Action>> {
    let lang = app.lang;
    match key.code {
        KeyCode::Esc | KeyCode::Backspace => app.page = Page::Shell,
        KeyCode::Up => app.shell_theme_index = app.shell_theme_index.saturating_sub(1),
        KeyCode::Down => {
            app.shell_theme_index = (app.shell_theme_index + 1).min(THEMES.len() - 1)
        }
        KeyCode::Enter => {
            let theme = THEMES[app.shell_theme_index].0;
            app.selected_theme = theme.to_string();
            match crate::modules::shell::set_theme(theme) {
                Ok(()) => {
                    app.status_msg = match lang {
                        Lang::Chinese => format!("✅ 主题已设置为: {}", theme),
                        Lang::English => format!("✅ Theme set to: {}", theme),
                    };
                }
                Err(e) => app.status_msg = i18n::msg_fail(lang, "theme", &e.to_string()),
            }
            app.page = Page::Shell;
        }
        _ => {}
    }
    Ok(None)
}

fn handle_plugins(app: &mut App, key: KeyEvent) -> anyhow::Result<Option<Action>> {
    let lang = app.lang;
    match key.code {
        KeyCode::Esc | KeyCode::Backspace => {
            if !app.selected_plugins.is_empty() {
                match crate::modules::shell::set_plugins(&app.selected_plugins) {
                    Ok(()) => {
                        app.status_msg = match lang {
                            Lang::Chinese => {
                                format!("✅ 已配置 {} 个插件", app.selected_plugins.len())
                            }
                            Lang::English => {
                                format!("✅ {} plugins configured", app.selected_plugins.len())
                            }
                        };
                    }
                    Err(e) => app.status_msg = i18n::msg_fail(lang, "plugins", &e.to_string()),
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
            if !app.selected_plugins.is_empty() {
                match crate::modules::shell::set_plugins(&app.selected_plugins) {
                    Ok(()) => {
                        app.status_msg = match lang {
                            Lang::Chinese => {
                                format!("✅ 已配置 {} 个插件", app.selected_plugins.len())
                            }
                            Lang::English => {
                                format!("✅ {} plugins configured", app.selected_plugins.len())
                            }
                        };
                    }
                    Err(e) => app.status_msg = i18n::msg_fail(lang, "plugins", &e.to_string()),
                }
            }
            app.page = Page::Shell;
        }
        _ => {}
    }
    Ok(None)
}

fn handle_docker(app: &mut App, key: KeyEvent) -> anyhow::Result<Option<Action>> {
    let lang = app.lang;
    let max = i18n::docker_menu(lang).len();
    match key.code {
        KeyCode::Esc | KeyCode::Backspace => {
            app.page = Page::MainMenu;
            app.docker_index = 0;
        }
        KeyCode::Up => app.docker_index = app.docker_index.saturating_sub(1),
        KeyCode::Down => app.docker_index = (app.docker_index + 1).min(max - 1),
        KeyCode::Enter => {
            let (before, after): (&str, String) = match (lang, app.docker_index) {
                (Lang::Chinese, 0) => ("正在安装 Docker...", "✅ Docker 安装成功".into()),
                (Lang::English, 0) => ("Installing Docker...", "✅ Docker installed".into()),
                (Lang::Chinese, 1) => ("正在安装 Docker Compose...", "✅ Docker Compose 安装成功".into()),
                (Lang::English, 1) => ("Installing Docker Compose...", "✅ Docker Compose installed".into()),
                (Lang::Chinese, 2) => ("正在配置 docker 用户组...", "✅ 已将当前用户加入 docker 组 (重新登录生效)".into()),
                (Lang::English, 2) => ("Configuring docker group...", "✅ Added user to docker group (re-login required)".into()),
                (Lang::Chinese, 3) => ("正在启动 Docker 服务...", "✅ Docker 服务已启动并设为开机自启".into()),
                (Lang::English, 3) => ("Starting Docker...", "✅ Docker service started and enabled".into()),
                _ => return Ok(None),
            };
            app.status_msg = before.into();
            let idx = app.docker_index;
            return Ok(Some(Action::Execute(Box::new(move |terminal| {
                run_in_terminal(terminal, || match idx {
                    0 => crate::modules::docker::install_docker(),
                    1 => crate::modules::docker::install_compose(),
                    2 => crate::modules::docker::add_user_to_docker_group(),
                    3 => crate::modules::docker::start_docker_service(),
                    _ => Ok(()),
                })?;
                Ok(after)
            }))));
        }
        _ => {}
    }
    Ok(None)
}

fn handle_ssh(app: &mut App, key: KeyEvent) -> anyhow::Result<Option<Action>> {
    let lang = app.lang;
    let max = i18n::ssh_menu(lang).len();
    match key.code {
        KeyCode::Esc | KeyCode::Backspace => {
            app.last_pubkey.clear();
            app.page = Page::MainMenu;
            app.ssh_index = 0;
        }
        KeyCode::Up => app.ssh_index = app.ssh_index.saturating_sub(1),
        KeyCode::Down => app.ssh_index = (app.ssh_index + 1).min(max - 1),
        KeyCode::Enter => {
            let email = format!(
                "{}@localhost",
                std::env::var("USER").unwrap_or_else(|_| "user".into())
            );
            match app.ssh_index {
                0 => match crate::modules::ssh::generate_ed25519(&email) {
                    Ok(pubkey) => {
                        app.ssh_key_exists = true;
                        app.last_pubkey = pubkey;
                        app.status_msg = match lang {
                            Lang::Chinese => "✅ Ed25519 密钥已生成".into(),
                            Lang::English => "✅ Ed25519 key generated".into(),
                        };
                    }
                    Err(e) => {
                        app.status_msg =
                            i18n::msg_fail(lang, "Ed25519 key", &e.to_string())
                    }
                },
                1 => match crate::modules::ssh::generate_rsa(&email) {
                    Ok(pubkey) => {
                        app.ssh_key_exists = true;
                        app.last_pubkey = pubkey;
                        app.status_msg = match lang {
                            Lang::Chinese => "✅ RSA 4096 密钥已生成".into(),
                            Lang::English => "✅ RSA 4096 key generated".into(),
                        };
                    }
                    Err(e) => {
                        app.status_msg =
                            i18n::msg_fail(lang, "RSA key", &e.to_string())
                    }
                },
                2 => match crate::modules::ssh::read_public_key() {
                    Ok(pubkey) => app.last_pubkey = pubkey,
                    Err(e) => app.status_msg = i18n::msg_fail(lang, "read key", &e.to_string()),
                },
                _ => {}
            }
        }
        _ => {}
    }
    Ok(None)
}

fn handle_ssh_server(app: &mut App, key: KeyEvent) -> anyhow::Result<Option<Action>> {
    let lang = app.lang;
    let max = i18n::ssh_server_menu(lang).len();
    match key.code {
        KeyCode::Esc | KeyCode::Backspace => {
            app.page = Page::MainMenu;
            app.ssh_server_index = 0;
        }
        KeyCode::Up => app.ssh_server_index = app.ssh_server_index.saturating_sub(1),
        KeyCode::Down => {
            app.ssh_server_index = (app.ssh_server_index + 1).min(max - 1)
        }
        KeyCode::Enter => match app.ssh_server_index {
            0 => {
                if !app.sshd_installed {
                    app.status_msg = i18n::msg_installing(lang, "OpenSSH Server");
                    let success_msg = i18n::msg_success(lang, "OpenSSH Server");
                    return Ok(Some(Action::Execute(Box::new(move |terminal| {
                        run_in_terminal(terminal, || {
                            crate::modules::ssh_server::install()
                        })?;
                        Ok(success_msg)
                    }))));
                }
            }
            1 => {
                if !app.sshd_root_disabled {
                    let msg = match lang {
                        Lang::Chinese => "正在配置禁止 root 登录...",
                        Lang::English => "Configuring disable root login...",
                    };
                    app.status_msg = msg.into();
                    let done_msg = match lang {
                        Lang::Chinese => "✅ 已禁止 root 远程登录".to_string(),
                        Lang::English => "✅ Root remote login disabled".to_string(),
                    };
                    return Ok(Some(Action::Execute(Box::new(move |terminal| {
                        run_in_terminal(terminal, || {
                            crate::modules::ssh_server::disable_root_login()
                        })?;
                        Ok(done_msg)
                    }))));
                }
            }
            2 => {
                let msg = match lang {
                    Lang::Chinese => "正在启动 SSH 服务...",
                    Lang::English => "Starting SSH service...",
                };
                app.status_msg = msg.into();
                let done_msg = match lang {
                    Lang::Chinese => "✅ SSH 服务已启动并设为开机自启".to_string(),
                    Lang::English => "✅ SSH service started and enabled".to_string(),
                };
                return Ok(Some(Action::Execute(Box::new(move |terminal| {
                    run_in_terminal(terminal, || {
                        crate::modules::ssh_server::start_service()
                    })?;
                    Ok(done_msg)
                }))));
            }
            _ => {}
        },
        _ => {}
    }
    Ok(None)
}

fn handle_tools(app: &mut App, key: KeyEvent) -> anyhow::Result<Option<Action>> {
    let lang = app.lang;
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
                app.status_msg = match lang {
                    Lang::Chinese => "请先选择要安装的工具".into(),
                    Lang::English => "Please select tools to install".into(),
                };
                return Ok(None);
            }

            let count = selected.len();
            app.status_msg = i18n::msg_installing(lang, &format!("{} tools", count));
            let selected_owned: Vec<String> =
                selected.iter().map(|s| s.to_string()).collect();
            return Ok(Some(Action::Execute(Box::new(move |terminal| {
                run_in_terminal(terminal, || {
                    let refs: Vec<&str> =
                        selected_owned.iter().map(|s| s.as_str()).collect();
                    crate::modules::tools::install_tools(&refs)
                })?;
                Ok(match lang {
                    Lang::Chinese => format!("✅ {} 个工具安装成功", count),
                    Lang::English => format!("✅ {} tools installed", count),
                })
            }))));
        }
        _ => {}
    }
    Ok(None)
}

fn handle_vim(
    terminal: &mut Term,
    app: &mut App,
    key: KeyEvent,
) -> anyhow::Result<Option<Action>> {
    let lang = app.lang;
    let max = i18n::vim_menu(lang).len();
    match key.code {
        KeyCode::Esc | KeyCode::Backspace => {
            app.page = Page::MainMenu;
            app.vim_index = 0;
        }
        KeyCode::Up => app.vim_index = app.vim_index.saturating_sub(1),
        KeyCode::Down => app.vim_index = (app.vim_index + 1).min(max - 1),
        KeyCode::Enter => {
            match app.vim_index {
                0 => {
                    if !app.vim_installed {
                        app.status_msg = i18n::msg_installing(lang, "Vim");
                        let result = run_in_terminal(terminal, || {
                            crate::modules::vim::install_vim()
                        });
                        match result {
                            Ok(()) => {
                                app.vim_installed = true;
                                app.status_msg = i18n::msg_success(lang, "Vim");
                            }
                            Err(e) => {
                                app.status_msg =
                                    i18n::msg_fail(lang, "Vim", &e.to_string())
                            }
                        }
                    }
                }
                1 => {
                    if !app.vundle_installed {
                        if !app.vim_installed {
                            app.status_msg = match lang {
                                Lang::Chinese => "❌ 请先安装 Vim".into(),
                                Lang::English => "❌ Please install Vim first".into(),
                            };
                            return Ok(None);
                        }
                        app.status_msg = i18n::msg_installing(lang, "Vundle");
                        let result = run_in_terminal(terminal, || {
                            crate::modules::vim::install_vundle()
                        });
                        match result {
                            Ok(()) => {
                                app.vundle_installed = true;
                                app.status_msg = i18n::msg_success(lang, "Vundle");
                            }
                            Err(e) => {
                                app.status_msg =
                                    i18n::msg_fail(lang, "Vundle", &e.to_string())
                            }
                        }
                    }
                }
                2 => {
                    if !app.vundle_installed {
                        app.status_msg = match lang {
                            Lang::Chinese => "❌ 请先安装 Vundle".into(),
                            Lang::English => "❌ Please install Vundle first".into(),
                        };
                        return Ok(None);
                    }
                    app.page = Page::VimPlugins;
                    app.vim_plugin_index = 0;
                }
                _ => {}
            }
        }
        _ => {}
    }
    Ok(None)
}

fn handle_vim_plugins(app: &mut App, key: KeyEvent) -> anyhow::Result<Option<Action>> {
    let lang = app.lang;
    let plugin_count = crate::modules::vim::VIM_PLUGINS.len();
    match key.code {
        KeyCode::Esc | KeyCode::Backspace => {
            // Save on exit
            if !app.selected_vim_plugins.is_empty() {
                match crate::modules::vim::write_vimrc(&app.selected_vim_plugins) {
                    Ok(()) => {
                        app.status_msg = match lang {
                            Lang::Chinese => format!(
                                "✅ 已配置 {} 个 Vim 插件 (运行 :PluginInstall 安装)",
                                app.selected_vim_plugins.len()
                            ),
                            Lang::English => format!(
                                "✅ {} Vim plugins configured (run :PluginInstall to install)",
                                app.selected_vim_plugins.len()
                            ),
                        };
                    }
                    Err(e) => app.status_msg = i18n::msg_fail(lang, "vimrc", &e.to_string()),
                }
            }
            app.page = Page::Vim;
        }
        KeyCode::Up => app.vim_plugin_index = app.vim_plugin_index.saturating_sub(1),
        KeyCode::Down => {
            app.vim_plugin_index = (app.vim_plugin_index + 1).min(plugin_count - 1)
        }
        KeyCode::Char(' ') => {
            let idx = app.vim_plugin_index;
            if let Some(pos) = app.selected_vim_plugins.iter().position(|&i| i == idx) {
                app.selected_vim_plugins.remove(pos);
            } else {
                app.selected_vim_plugins.push(idx);
            }
            app.vim_plugin_index = (app.vim_plugin_index + 1).min(plugin_count - 1);
        }
        KeyCode::Enter => {
            if !app.selected_vim_plugins.is_empty() {
                match crate::modules::vim::write_vimrc(&app.selected_vim_plugins) {
                    Ok(()) => {
                        app.status_msg = match lang {
                            Lang::Chinese => format!(
                                "✅ 已配置 {} 个 Vim 插件 (运行 :PluginInstall 安装)",
                                app.selected_vim_plugins.len()
                            ),
                            Lang::English => format!(
                                "✅ {} Vim plugins configured (run :PluginInstall to install)",
                                app.selected_vim_plugins.len()
                            ),
                        };
                    }
                    Err(e) => app.status_msg = i18n::msg_fail(lang, "vimrc", &e.to_string()),
                }
            }
            app.page = Page::Vim;
        }
        _ => {}
    }
    Ok(None)
}

fn handle_nvm(_terminal: &mut Term, app: &mut App, key: KeyEvent) -> anyhow::Result<Option<Action>> {
    let lang = app.lang;
    let max = i18n::nvm_menu(lang).len();
    match key.code {
        KeyCode::Esc | KeyCode::Backspace => {
            app.page = Page::MainMenu;
            app.nvm_index = 0;
        }
        KeyCode::Up => app.nvm_index = app.nvm_index.saturating_sub(1),
        KeyCode::Down => app.nvm_index = (app.nvm_index + 1).min(max - 1),
        KeyCode::Enter => match app.nvm_index {
            0 => {
                app.status_msg = i18n::msg_installing(lang, "nvm");
                return Ok(Some(Action::Execute(Box::new(move |terminal| {
                    run_in_terminal(terminal, || crate::modules::nvm::install_nvm())?;
                    Ok(i18n::msg_success(lang, "nvm"))
                }))));
            }
            1 => {
                if !app.nvm_installed {
                    app.status_msg = match lang {
                        Lang::Chinese => "❌ 请先安装 nvm".into(),
                        Lang::English => "❌ Please install nvm first".into(),
                    };
                    return Ok(None);
                }
                app.status_msg = i18n::msg_installing(lang, "Node.js LTS");
                return Ok(Some(Action::Execute(Box::new(move |terminal| {
                    run_in_terminal(terminal, || crate::modules::nvm::install_node_lts())?;
                    Ok(i18n::msg_success(lang, "Node.js LTS"))
                }))));
            }
            2 => {
                app.status_msg = match lang {
                    Lang::Chinese => "正在配置 Shell 集成...".into(),
                    Lang::English => "Configuring shell integration...".into(),
                };
                return Ok(Some(Action::Execute(Box::new(move |terminal| {
                    run_in_terminal(terminal, || crate::modules::nvm::ensure_shell_integration())?;
                    Ok(match lang {
                        Lang::Chinese => "✅ Shell 集成配置完成 (重新打开终端生效)".into(),
                        Lang::English => "✅ Shell integration configured (reopen terminal to apply)".into(),
                    })
                }))));
            }
            _ => {}
        },
        _ => {}
    }
    Ok(None)
}

fn handle_locale(app: &mut App, key: KeyEvent) -> anyhow::Result<Option<Action>> {
    let lang = app.lang;
    let max = i18n::locale_menu(lang).len();
    match key.code {
        KeyCode::Esc | KeyCode::Backspace => {
            app.page = Page::MainMenu;
            app.locale_index = 0;
        }
        KeyCode::Up => app.locale_index = app.locale_index.saturating_sub(1),
        KeyCode::Down => app.locale_index = (app.locale_index + 1).min(max - 1),
        KeyCode::Enter => {
            let (before, after): (&str, String) = match (lang, app.locale_index) {
                (Lang::Chinese, 0) => ("正在配置中文 locale...", "✅ 中文 locale 配置成功".into()),
                (Lang::English, 0) => ("Configuring Chinese locale...", "✅ Chinese locale configured".into()),
                (Lang::Chinese, 1) => ("正在安装中文字体...", "✅ 中文字体安装成功".into()),
                (Lang::English, 1) => ("Installing Chinese fonts...", "✅ Chinese fonts installed".into()),
                (Lang::Chinese, 2) => ("正在安装 Fcitx5 输入法...", "✅ Fcitx5 输入法安装成功 (重新登录生效)".into()),
                (Lang::English, 2) => ("Installing Fcitx5 input method...", "✅ Fcitx5 installed (re-login required)".into()),
                _ => return Ok(None),
            };
            app.status_msg = before.into();
            let idx = app.locale_index;
            return Ok(Some(Action::Execute(Box::new(move |terminal| {
                run_in_terminal(terminal, || match idx {
                    0 => crate::modules::locale::configure_locale(),
                    1 => crate::modules::locale::install_cjk_fonts(),
                    2 => crate::modules::locale::install_fcitx5(),
                    _ => Ok(()),
                })?;
                Ok(after)
            }))));
        }
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
