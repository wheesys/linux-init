use crate::config::Config;
use crate::distro::Distro;
use crate::i18n::Lang;

#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum Page {
    LangSelect,
    MainMenu,
    Shell,
    ShellZshTheme,
    ShellZshPlugins,
    Docker,
    Ssh,
    SshServer,
    Tools,
    Vim,
    VimPlugins,
    VimOptimize,
    Nvm,
    NvmNodeVersion,
    Locale,
    Status(Box<StatusData>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct StatusData {
    pub title: String,
    pub lines: Vec<String>,
    pub back: Page,
}

pub struct App {
    pub distro: Distro,
    pub lang: Lang,
    pub lang_selected: bool,
    pub config: Config,
    pub running: bool,
    pub page: Page,
    pub status_msg: String,
    pub input_buf: String,

    // -- lang select --
    pub lang_index: usize,

    // -- main menu --
    pub menu_index: usize,

    // -- shell --
    pub shell_index: usize,
    pub zsh_installed: bool,
    pub omz_installed: bool,
    pub omz_configured: bool,
    pub default_shell_set: bool,
    /// 用户在本次会话中选定的 shell（"zsh"/"bash"），用于 shell 未重启时的配置 fallback
    pub selected_shell: Option<String>,
    pub shell_theme_index: usize,
    pub selected_theme: String,
    pub plugin_index: usize,
    pub selected_plugins: Vec<String>,

    // -- docker --
    pub docker_index: usize,
    pub docker_installed: bool,
    pub compose_installed: bool,
    pub docker_user_configured: bool,
    pub docker_service_running: bool,

    // -- ssh key --
    pub ssh_index: usize,
    pub ed25519_exists: bool,
    pub rsa_exists: bool,
    pub last_pubkey: String,

    // -- ssh server --
    pub ssh_server_index: usize,
    pub sshd_installed: bool,
    pub sshd_root_disabled: bool,
    pub sshd_running: bool,

    // -- tools --
    pub tool_index: usize,
    pub selected_tools: Vec<bool>,

    // -- vim --
    pub vim_index: usize,
    pub vim_installed: bool,
    pub vundle_installed: bool,
    pub vim_plugin_index: usize,
    pub selected_vim_plugins: Vec<usize>,
    pub vim_opt_index: usize,
    pub selected_vim_opts: Vec<usize>,

    // -- nvm --
    pub nvm_index: usize,
    pub nvm_installed: bool,
    pub node_installed: bool,
    pub nvm_node_index: usize,

    // -- locale --
    pub locale_index: usize,
    pub locale_configured: bool,
    pub fonts_installed: bool,
    pub fcitx_installed: bool,
}

pub const THEMES: &[(&str, &str)] = &[
    ("robbyrussell", ""),
    ("af-magic", ""),
    ("agnoster", ""),
    ("bira", ""),
    ("clean", ""),
    ("dst", ""),
    ("gallifrey", ""),
    ("maran", ""),
    ("minimal", ""),
    ("powerlevel10k", ""),
    ("refined", ""),
    ("suvash", ""),
    ("ys", ""),
];

/// 每个主题对应的 prompt 示例（ASCII 模拟效果）
// 主题预览数据结构：每个主题包含多行，每行包含多个 (文本, 颜色) 片段
pub const THEME_PREVIEWS: &[(&str, &[&[(&str, &str)]])] = &[
    ("robbyrussell", &[
        &[("➜ ", "green"), ("~ ", "cyan"), ("git:(", "white"), ("master", "red"), (") ", "white"), ("✗", "red")],
        &[],
        &[("单行简洁提示，显示路径和 git 状态", "dim")],
    ]),
    ("af-magic", &[
        &[("┌─ ", "cyan"), ("~/projects", "blue"), ("  ── ", "cyan"), ("git:(", "white"), ("master", "red"), (") ─────┐", "cyan")],
        &[("└─$ ", "cyan")],
        &[],
        &[("双行边框提示，路径和 git 分支分离", "dim")],
    ]),
    ("agnoster", &[
        &[(" ", "black_bg"), ("user@host", "green_fg"), ("  ", "black_bg"), ("master", "blue_bg"), (" ±", "yellow_bg"), ("  ", "black_bg"), ("~/projects", "cyan_bg"), (" ", "black_bg")],
        &[],
        &[("Powerline 分段风格，紧凑信息密集", "dim")],
    ]),
    ("bira", &[
        &[("╭─ ", "cyan"), ("user@host", "green"), (" ─ [", "cyan"), ("~/projects", "blue"), ("] ─ (", "cyan"), ("master", "red"), (" ✗)", "yellow")],
        &[("╰─$ ", "cyan")],
        &[],
        &[("双行圆角边框，彩色信息丰富", "dim")],
    ]),
    ("clean", &[
        &[("/home/user/projects", "blue"), (" (", "white"), ("master", "red"), (") ", "white"), ("$ ", "green")],
        &[],
        &[("极简风格，仅路径和 git 符号", "dim")],
    ]),
    ("dst", &[
        &[("[", "white"), ("master", "red"), ("] ", "white"), ("~/projects ", "blue"), ("$ ", "green")],
        &[],
        &[("经典单行，git 分支在前", "dim")],
    ]),
    ("gallifrey", &[
        &[("╭─[", "yellow"), ("TARDIS", "blue"), ("] ", "yellow"), ("~/projects", "cyan")],
        &[("╰─$ ", "yellow")],
        &[],
        &[("Doctor Who 风格双行提示", "dim")],
    ]),
    ("maran", &[
        &[("user@host", "green"), (" ", "white"), ("~/projects", "blue"), (" (", "white"), ("master", "red"), (") ", "white"), ("$ ", "green")],
        &[],
        &[("简洁单行，用户@主机 + 路径", "dim")],
    ]),
    ("minimal", &[
        &[("⚡ ", "yellow"), ("~/projects ", "blue"), ("git:", "white"), ("master ", "red")],
        &[],
        &[("极简闪电风格，无多余符号", "dim")],
    ]),
    ("powerlevel10k", &[
        &[(" ", "blue_bg"), ("~/projects", "white_fg"), ("  ", "blue_bg"), ("master", "green_bg"), (" !2 ?5", "yellow_bg"), ("  ", "blue_bg"), ("12:34 ", "cyan_bg"), (" ", "blue_bg")],
        &[],
        &[("高度可定制，需运行配置向导", "dim")],
        &[("显示 git 变更统计、时间等", "dim")],
    ]),
    ("refined", &[
        &[("~/projects", "blue"), ("  ", "white"), ("master", "red"), (" $ ", "green")],
        &[],
        &[("精炼单行，无特殊符号", "dim")],
    ]),
    ("suvash", &[
        &[("➜ ", "green"), ("~/projects ", "blue"), ("git:(", "white"), ("master", "red"), (")  ", "white"), ("✗ ", "red")],
        &[],
        &[("带箭头的 git 状态提示", "dim")],
    ]),
    ("ys", &[
        &[("# ", "yellow"), ("user", "green"), (" @ ", "yellow"), ("host", "green"), (" in ", "yellow"), ("~/projects", "blue"), (" [", "white"), ("master", "red"), ("|", "white"), ("✚2…1", "yellow"), ("]", "white")],
        &[("12:34:56 ", "cyan"), ("$ ", "green")],
        &[],
        &[("信息丰富的双行，显示时间和变更", "dim")],
    ]),
];

pub const PLUGINS: &[(&str, &str)] = &[
    ("git", "内置"),
    ("zsh-autosuggestions", "第三方"),
    ("zsh-syntax-highlighting", "第三方"),
    ("docker", "内置"),
    ("docker-compose", "内置"),
    ("kubectl", "内置"),
    ("sudo", "内置"),
    ("extract", "内置"),
    ("colorize", "内置"),
    ("copypath", "内置"),
    ("z", "内置"),
    ("fzf", "第三方"),
    ("zsh-completions", "第三方"),
    ("you-should-use", "第三方"),
];

pub const TOOLS: &[(&str, &str, &str)] = &[
    ("git", "开发工具", ""),
    ("curl", "开发工具", ""),
    ("wget", "开发工具", ""),
    ("btop", "系统工具", ""),
    ("neovim", "开发工具", ""),
    ("tmux", "系统工具", ""),
    ("jq", "开发工具", ""),
    ("ripgrep", "CLI 增强", ""),
    ("fd", "CLI 增强", ""),
    ("bat", "CLI 增强", ""),
    ("eza", "CLI 增强", ""),
    ("trash-cli", "CLI 增强", ""),
    ("procs", "CLI 增强", ""),
    ("dust", "CLI 增强", ""),
    ("duf", "CLI 增强", ""),
    ("direnv", "开发工具", ""),
];

impl App {
    pub fn new(distro: Distro) -> Self {
        let config = Config::load();
        let home = Self::get_real_home();
        let zsh_installed = crate::distro::is_package_installed("zsh");
        let omz_installed = home.join(".oh-my-zsh").exists();
        let docker_installed = crate::distro::is_tool_installed("docker");
        let compose_installed = crate::distro::is_tool_installed("docker-compose");
        let docker_user_configured = crate::modules::docker::is_user_in_docker_group();
        let docker_service_running = crate::modules::docker::is_docker_running();
        let ed25519_exists = home.join(".ssh/id_ed25519.pub").exists();
        let rsa_exists = home.join(".ssh/id_rsa.pub").exists();
        let vim_installed = crate::distro::is_package_installed("vim");
        let vundle_installed = home.join(".vim/bundle/Vundle.vim").exists();

        let current_shell = std::env::var("SHELL").unwrap_or_default();
        let default_shell_set = current_shell.contains("zsh");

        let selected_theme = if omz_installed {
            Self::read_current_theme(&home)
        } else {
            "robbyrussell".to_string()
        };

        let selected_plugins = if omz_installed {
            Self::read_current_plugins(&home)
        } else {
            vec!["git".to_string()]
        };

        // Detect language from config or system locale
        let (lang, lang_selected, page) = if let Some(ref lang_str) = config.language {
            let lang = match lang_str.as_str() {
                "zh" => Lang::Chinese,
                _ => Lang::English,
            };
            (lang, true, Page::MainMenu)
        } else {
            let sys_lang = std::env::var("LANG").unwrap_or_default();
            let default_lang = if sys_lang.starts_with("zh") {
                Lang::Chinese
            } else {
                Lang::English
            };
            (default_lang, false, Page::LangSelect)
        };

        Self {
            distro,
            lang,
            lang_selected,
            config,
            running: true,
            page,
            status_msg: String::new(),
            input_buf: String::new(),

            lang_index: if lang == Lang::Chinese { 0 } else { 1 },
            menu_index: 0,

            shell_index: 0,
            zsh_installed,
            omz_installed,
            omz_configured: omz_installed,
            default_shell_set,
            selected_shell: None,
            shell_theme_index: 0,
            selected_theme,
            plugin_index: 0,
            selected_plugins,

            docker_index: 0,
            docker_installed,
            compose_installed,
            docker_user_configured,
            docker_service_running,

            ssh_index: 0,
            ed25519_exists,
            rsa_exists,
            last_pubkey: String::new(),

            ssh_server_index: 0,
            sshd_installed: crate::modules::ssh_server::is_installed(),
            sshd_root_disabled: crate::modules::ssh_server::is_root_login_disabled(),
            sshd_running: crate::modules::ssh_server::is_running(),

            tool_index: 0,
            selected_tools: vec![false; TOOLS.len()],

            vim_index: 0,
            vim_installed,
            vundle_installed,
            vim_plugin_index: 0,
            selected_vim_plugins: vec![],
            vim_opt_index: 0,
            selected_vim_opts: vec![],

            nvm_index: 0,
            nvm_installed: crate::modules::nvm::is_nvm_installed(),
            node_installed: crate::modules::nvm::installed_node_version().is_some(),
            nvm_node_index: 0,

            locale_index: 0,
            locale_configured: crate::modules::locale::is_locale_configured(),
            fonts_installed: crate::modules::locale::is_cjk_fonts_installed(),
            fcitx_installed: crate::modules::locale::is_fcitx_installed(),
        }
    }

    fn get_real_home() -> std::path::PathBuf {
        crate::utils::get_real_home().unwrap_or_else(|_| dirs::home_dir().unwrap_or_default())
    }

    fn read_current_theme(home: &std::path::Path) -> String {
        let zshrc = home.join(".zshrc");
        if let Ok(content) = std::fs::read_to_string(&zshrc) {
            for line in content.lines() {
                let trimmed = line.trim();
                if trimmed.starts_with("ZSH_THEME=") {
                    return trimmed
                        .trim_start_matches("ZSH_THEME=")
                        .trim_matches('"')
                        .trim_matches('\'')
                        .to_string();
                }
            }
        }
        "robbyrussell".to_string()
    }

    fn read_current_plugins(home: &std::path::Path) -> Vec<String> {
        let zshrc = home.join(".zshrc");
        if let Ok(content) = std::fs::read_to_string(&zshrc) {
            for line in content.lines() {
                let trimmed = line.trim();
                if trimmed.starts_with("plugins=") {
                    let raw = trimmed
                        .trim_start_matches("plugins=")
                        .trim_matches('(')
                        .trim_matches(')')
                        .trim_matches('"');
                    return raw.split_whitespace().map(|s| s.to_string()).collect();
                }
            }
        }
        vec!["git".to_string()]
    }
}
