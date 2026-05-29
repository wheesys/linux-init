use crate::distro;

pub fn install_tools(selected: &[&str]) -> anyhow::Result<()> {
    let mut packages: Vec<&str> = Vec::new();

    for tool in selected {
        if let Some(pkg) = distro::package_name(tool) {
            if !distro::is_package_installed(pkg) {
                packages.push(pkg);
            }
        }
    }

    if packages.is_empty() {
        return Ok(());
    }

    distro::install_packages(&packages)?;
    Ok(())
}

pub fn get_tool_status(tool: &str) -> bool {
    if let Some(pkg) = distro::package_name(tool) {
        distro::is_package_installed(pkg)
    } else {
        false
    }
}
