use std::fs;
use std::process::{Command, Stdio};

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
    // 确保依赖命令存在
    crate::utils::ensure_command("git")?;
    
    let home = get_real_home()?;
    let vundle_dir = home.join(".vim/bundle/Vundle.vim");

    if vundle_dir.exists() {
        return Ok(());
    }

    let status = Command::new("git")
        .args([
            "clone",
            "https://github.com/VundleVim/Vundle.vim.git",
            vundle_dir.to_str().unwrap(),
        ])
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()?;

    if !status.success() {
        anyhow::bail!("Vundle 安装失败");
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

pub fn write_vimrc(selected_plugins: &[usize]) -> anyhow::Result<()> {
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

    // Basic settings
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

    // NERDTree config
    if selected_plugins.contains(&0) {
        content.push_str("\n\" NERDTree\n");
        content.push_str("nnoremap <C-n> :NERDTreeToggle<CR>\n");
    }

    // vim-airline
    if selected_plugins.contains(&1) {
        content.push_str("\n\" vim-airline\n");
        content.push_str("let g:airline_powerline_fonts = 1\n");
    }

    // ctrlp
    if selected_plugins.contains(&9) {
        content.push_str("\n\" CtrlP\n");
        content.push_str("let g:ctrlp_map = '<c-p>'\n");
    }

    // easymotion
    if selected_plugins.contains(&8) {
        content.push_str("\n\" vim-easymotion\n");
        content.push_str("let g:EasyMotion_smartcase = 1\n");
    }

    fs::write(&vimrc_path, content)?;
    Ok(())
}

#[allow(dead_code)]
pub fn install_plugin_bundle() -> anyhow::Result<()> {
    let status = Command::new("vim")
        .args(["+PluginInstall", "+qall"])
        .stdin(Stdio::null())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()?;

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
