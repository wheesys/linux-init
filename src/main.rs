mod app;
mod config;
mod distro;
mod i18n;
mod modules;
mod ui;
mod utils;

use anyhow::Result;
use app::App;
use ui::Action;

fn main() -> Result<()> {
    let distro = distro::detect();
    let mut terminal = ui::setup_terminal()?;
    let mut app = App::new(distro);

    let result = run_app(&mut terminal, &mut app);

    ui::restore_terminal(&mut terminal)?;

    if let Err(ref e) = result {
        eprintln!("Error: {}", e);
    }

    result
}

fn run_app(terminal: &mut ui::Term, app: &mut App) -> Result<()> {
    while app.running {
        terminal.draw(|frame| ui::render(frame, app))?;

        if let Some(action) = ui::handle_event(terminal, app)? {
            match action {
                Action::Quit => {
                    app.running = false;
                }
                Action::Execute(f) => match f(terminal) {
                    Ok(msg) => app.status_msg = msg,
                    Err(e) => app.status_msg = format!("❌ {}", e),
                },
            }
            refresh_state(app);
        }
    }
    Ok(())
}

fn refresh_state(app: &mut App) {
    app.zsh_installed = distro::is_package_installed("zsh");
    let home = utils::get_real_home().unwrap_or_else(|_| dirs::home_dir().unwrap_or_default());
    app.omz_installed = home.join(".oh-my-zsh").exists();
    app.docker_installed = distro::is_package_installed("docker");
    app.compose_installed = distro::is_package_installed("docker-compose");
    app.docker_user_configured = modules::docker::is_user_in_docker_group();
    app.docker_service_running = modules::docker::is_docker_running();
    app.ed25519_exists = modules::ssh::has_ed25519_key();
    app.rsa_exists = modules::ssh::has_rsa_key();
    app.sshd_installed = modules::ssh_server::is_installed();
    app.sshd_root_disabled = modules::ssh_server::is_root_login_disabled();
    app.sshd_running = modules::ssh_server::is_running();
    app.locale_configured = modules::locale::is_locale_configured();
    app.fcitx_installed = modules::locale::is_fcitx_installed();
    app.fonts_installed = distro::is_package_installed("noto-fonts-cjk");
    app.vim_installed = distro::is_package_installed("vim");
    app.vundle_installed = home.join(".vim/bundle/Vundle.vim").exists();
    app.nvm_installed = modules::nvm::is_nvm_installed();
    app.node_installed = modules::nvm::installed_node_version().is_some();

    let current_shell = std::env::var("SHELL").unwrap_or_default();
    app.default_shell_set = current_shell.contains("zsh");

    // Sync config with actual state
    let mut config_changed = false;

    if app.zsh_installed && !app.config.completed.zsh_installed {
        app.config.completed.zsh_installed = true;
        config_changed = true;
    }
    if app.omz_installed && !app.config.completed.omz_installed {
        app.config.completed.omz_installed = true;
        config_changed = true;
    }
    if app.docker_installed && !app.config.completed.docker_installed {
        app.config.completed.docker_installed = true;
        config_changed = true;
    }
    if app.compose_installed && !app.config.completed.docker_compose_installed {
        app.config.completed.docker_compose_installed = true;
        config_changed = true;
    }
    if app.docker_user_configured && !app.config.completed.docker_user_configured {
        app.config.completed.docker_user_configured = true;
        config_changed = true;
    }
    if app.docker_service_running && !app.config.completed.docker_service_running {
        app.config.completed.docker_service_running = true;
        config_changed = true;
    }
    if (app.ed25519_exists || app.rsa_exists) && !app.config.completed.ssh_key_generated {
        app.config.completed.ssh_key_generated = true;
        config_changed = true;
    }
    if app.sshd_installed && !app.config.completed.ssh_server_installed {
        app.config.completed.ssh_server_installed = true;
        config_changed = true;
    }
    if app.sshd_root_disabled && !app.config.completed.ssh_server_configured {
        app.config.completed.ssh_server_configured = true;
        config_changed = true;
    }
    if app.sshd_running && !app.config.completed.ssh_server_running {
        app.config.completed.ssh_server_running = true;
        config_changed = true;
    }
    if app.vim_installed && !app.config.completed.vim_installed {
        app.config.completed.vim_installed = true;
        config_changed = true;
    }
    if app.vundle_installed && !app.config.completed.vundle_installed {
        app.config.completed.vundle_installed = true;
        config_changed = true;
    }
    if app.nvm_installed && !app.config.completed.nvm_installed {
        app.config.completed.nvm_installed = true;
        config_changed = true;
    }
    if app.node_installed && !app.config.completed.node_installed {
        app.config.completed.node_installed = true;
        config_changed = true;
    }
    if app.locale_configured && !app.config.completed.chinese_locale_configured {
        app.config.completed.chinese_locale_configured = true;
        config_changed = true;
    }
    if app.fonts_installed && !app.config.completed.chinese_fonts_installed {
        app.config.completed.chinese_fonts_installed = true;
        config_changed = true;
    }
    if app.fcitx_installed && !app.config.completed.fcitx5_installed {
        app.config.completed.fcitx5_installed = true;
        config_changed = true;
    }

    if config_changed {
        let _ = app.config.save();
    }
}
