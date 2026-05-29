mod app;
mod distro;
mod i18n;
mod modules;
mod ui;

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
    let home = dirs::home_dir().unwrap_or_default();
    app.omz_installed = home.join(".oh-my-zsh").exists();
    app.docker_installed = distro::is_package_installed("docker");
    app.compose_installed = distro::is_package_installed("docker-compose");
    app.docker_user_configured = modules::docker::is_user_in_docker_group();
    app.ssh_key_exists = modules::ssh::has_ssh_key();
    app.sshd_installed = modules::ssh_server::is_installed();
    app.sshd_root_disabled = modules::ssh_server::is_root_login_disabled();
    app.locale_configured = modules::locale::is_locale_configured();
    app.fcitx_installed = modules::locale::is_fcitx_installed();
    app.fonts_installed = distro::is_package_installed("noto-fonts-cjk");
    app.vim_installed = distro::is_package_installed("vim");
    app.vundle_installed = home.join(".vim/bundle/Vundle.vim").exists();
    app.nvm_installed = modules::nvm::is_nvm_installed();
    app.node_installed = modules::nvm::installed_node_version().is_some();

    let current_shell = std::env::var("SHELL").unwrap_or_default();
    app.default_shell_set = current_shell.contains("zsh");
}
