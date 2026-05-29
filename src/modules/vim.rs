use std::fs;
use crate::utils::{get_real_home, DownloadLogger};

#[allow(dead_code)]
pub fn is_vim_installed() -> bool {
    crate::distro::is_package_installed("vim")
}

pub fn install_vim() -> anyhow::Result<()> {
    crate::distro::install_packages(&["vim"])
}

#[allow(dead_code)]
pub fn is_vundle_installed() -> bool {
    let home = match get_real_home() {
        Ok(h) => h,
        Err(_) => return false,
    };
    home.join(".vim/bundle/Vundle.vim").exists()
}

pub fn install_vundle() -> anyhow::Result<()> {
    crate::utils::ensure_command("git")?;

    let home = get_real_home()?;
    let vundle_dir = home.join(".vim/bundle/Vundle.vim");

    let mut log = DownloadLogger::new("vundle-install.log")?;
    log.log(&format!("Vundle dir: {:?}", vundle_dir))?;

    if vundle_dir.exists() {
        log.log("Vundle already exists, skipping")?;
        log.finish(true);
        return Ok(());
    }

    let sources = [
        ("GitHub", "https://github.com/VundleVim/Vundle.vim.git"),
        ("Gitee", "https://gitee.com/mirrors/Vundle.vim.git"),
    ];

    let mut cloned = false;
    for (name, repo) in &sources {
        log.log(&format!("Trying {} mirror...", name))?;
        if log.check_network(repo).is_err() {
            log.log(&format!("{} check failed, trying next...", name))?;
            continue;
        }

        let status = if let Ok(sudo_user) = std::env::var("SUDO_USER") {
            log.run_as_user(
                &format!("Clone from {}", name),
                &sudo_user,
                "git",
                &["clone", repo, vundle_dir.to_str().unwrap()],
            )?
        } else {
            log.run_download(
                &format!("Clone from {}", name),
                "git",
                &["clone", repo, vundle_dir.to_str().unwrap()],
            )?
        };

        if status.success() {
            cloned = true;
            break;
        }
        log.log(&format!("{} clone failed, trying next...", name))?;
        // Clean up partial clone
        let _ = std::fs::remove_dir_all(&vundle_dir);
    }

    log.fix_ownership(vundle_dir.to_str().unwrap_or(""));

    let installed = vundle_dir.exists();
    log.finish(installed && cloned);

    if !cloned {
        anyhow::bail!("Vundle 安装失败，所有下载源均失败");
    }
    Ok(())
}

pub const VIM_PLUGINS: &[(&str, &str)] = &[
    ("nerdtree", "preservim/nerdtree"),
    ("vim-airline", "vim-airline/vim-airline"),
    ("vim-fugitive", "tpope/vim-fugitive"),
    ("nerdcommenter", "preservim/nerdcommenter"),
    ("vim-surround", "tpope/vim-surround"),
    ("vim-commentary", "tpope/vim-commentary"),
    ("auto-pairs", "jiangmiao/auto-pairs"),
    ("vim-gitgutter", "airblade/vim-gitgutter"),
    ("vim-easymotion", "easymotion/vim-easymotion"),
    ("ctrlp.vim", "ctrlpvim/ctrlp.vim"),
    ("vim-markdown", "preservim/vim-markdown"),
    ("tagbar", "preservim/tagbar"),
];

pub const VIM_OPTS: &[(&str, &[&str])] = &[
    ("mouse", &["set mouse=a"]),
    ("scrolloff", &["set scrolloff=5"]),
    ("laststatus", &["set laststatus=2"]),
    ("ignorecase", &["set ignorecase"]),
    ("fileformat", &["set fileformat=unix"]),
    ("cindent", &["set cindent"]),
    ("autoread", &["set autoread"]),
    ("whichwrap", &["set whichwrap+=<,>,h,l"]),
    ("matchtime", &["set matchtime=5"]),
    ("selection", &["set selection=exclusive", "set selectmode=mouse,key"]),
    ("guioptions", &["set guioptions-=r", "set guioptions-=L", "set guioptions-=b"]),
    ("showtabline", &["set showtabline=0"]),
];

pub fn write_vimrc(selected_plugins: &[usize], selected_opts: &[usize]) -> anyhow::Result<()> {
    let home = get_real_home()?;
    let vimrc_path = home.join(".vimrc");

    let mut content = String::new();
    content.push_str("set nocompatible\n");
    content.push_str("filetype off\n\n");
    content.push_str("set rtp+=~/.vim/bundle/Vundle.vim\n");
    content.push_str("call vundle#begin()\n\n");
    content.push_str("Plugin 'VundleVim/Vundle.vim'\n\n");

    for &idx in selected_plugins {
        if let Some((_, repo)) = VIM_PLUGINS.get(idx) {
            content.push_str(&format!("Plugin '{}'\n", repo));
        }
    }

    content.push_str("\ncall vundle#end()\n");
    content.push_str("filetype plugin indent on\n\n");

    content.push_str("syntax on\n");
    content.push_str("set number\n");
    content.push_str("set relativenumber\n");
    content.push_str("set cursorline\n");
    content.push_str("set showmatch\n");
    content.push_str("set hlsearch\n");
    content.push_str("set incsearch\n");
    content.push_str("set tabstop=4\n");
    content.push_str("set shiftwidth=4\n");
    content.push_str("set expandtab\n");
    content.push_str("set autoindent\n");
    content.push_str("set smartindent\n");
    content.push_str("set encoding=utf-8\n");
    content.push_str("set termencoding=utf-8\n");
    content.push_str("set backspace=indent,eol,start\n");

    if selected_plugins.contains(&0) {
        content.push_str("\n\" NERDTree\n");
        content.push_str("nnoremap <C-n> :NERDTreeToggle<CR>\n");
    }
    if selected_plugins.contains(&1) {
        content.push_str("\n\" vim-airline\n");
        content.push_str("let g:airline_powerline_fonts = 1\n");
    }
    if selected_plugins.contains(&9) {
        content.push_str("\n\" CtrlP\n");
        content.push_str("let g:ctrlp_map = '<c-p>'\n");
    }
    if selected_plugins.contains(&8) {
        content.push_str("\n\" vim-easymotion\n");
        content.push_str("let g:EasyMotion_smartcase = 1\n");
    }

    // --- 优化设置 ---
    if !selected_opts.is_empty() {
        content.push_str("\n\" ===== Vim Optimizations =====\n");
        for &idx in selected_opts {
            if let Some((key, lines)) = VIM_OPTS.get(idx) {
                content.push_str(&format!("\" {}\n", key));
                for line in *lines {
                    content.push_str(line);
                    content.push('\n');
                }
            }
        }
    }

    fs::write(&vimrc_path, content)?;
    Ok(())
}

#[allow(dead_code)]
pub fn install_plugin_bundle() -> anyhow::Result<()> {
    crate::utils::ensure_command("vim")?;

    let mut log = DownloadLogger::new("vim-plugins-install.log")?;

    log.check_network("https://github.com")?;

    log.log("Running vim +PluginInstall +qall")?;
    let status = if let Ok(sudo_user) = std::env::var("SUDO_USER") {
        log.run_as_user("PluginInstall", &sudo_user, "vim", &["+PluginInstall", "+qall"])?
    } else {
        log.run_download("PluginInstall", "vim", &["+PluginInstall", "+qall"])?
    };

    log.finish(status.success());

    if !status.success() {
        anyhow::bail!("Vim 插件安装失败");
    }
    Ok(())
}

#[allow(dead_code)]
pub fn get_installed_plugin_count() -> usize {
    let home = match get_real_home() {
        Ok(h) => h,
        Err(_) => return 0,
    };
    let bundle_dir = home.join(".vim/bundle");
    if !bundle_dir.exists() {
        return 0;
    }
    fs::read_dir(&bundle_dir)
        .map(|entries| {
            entries
                .filter_map(|e| e.ok())
                .filter(|e| e.file_name() != "Vundle.vim")
                .count()
        })
        .unwrap_or(0)
}

pub fn clear_vim() -> anyhow::Result<()> {
    let home = crate::utils::get_real_home()?;
    let _ = std::fs::remove_file(home.join(".vimrc"));
    let vim_dir = home.join(".vim");
    if vim_dir.exists() {
        std::fs::remove_dir_all(&vim_dir)?;
    }
    crate::distro::uninstall_packages(&["vim"])?;
    Ok(())
}
