use anyhow::{Context, bail};
use std::cmp::Ordering;
use std::fs;
use std::path::{Path, PathBuf};

const LIBRARY_NAME: &str = "libwikiquote_fetcher.so";
const VERSION: &str = env!("CARGO_PKG_VERSION");

fn version_numbers(version: &str) -> Option<Vec<u64>> {
    version
        .split_once('-')
        .map_or(version, |(stable, _)| stable)
        .split('.')
        .map(str::parse)
        .collect::<Result<Vec<_>, _>>()
        .ok()
}

fn compare_versions(left: &str, right: &str) -> Option<Ordering> {
    let mut left = version_numbers(left)?;
    let mut right = version_numbers(right)?;
    let width = left.len().max(right.len());
    left.resize(width, 0);
    right.resize(width, 0);
    Some(left.cmp(&right))
}

fn installed_version(directory: &Path) -> Option<(String, PathBuf)> {
    let link = directory.join(LIBRARY_NAME);
    let target = fs::read_link(&link).ok()?;
    let file_name = target.file_name()?.to_str()?;
    let version = file_name.strip_prefix(&format!("{LIBRARY_NAME}."))?;
    Some((version.to_string(), link))
}

fn user_library_dir() -> anyhow::Result<PathBuf> {
    let home = std::env::var_os("HOME").context("HOME is not set")?;
    Ok(PathBuf::from(home).join(".local/lib"))
}

fn release_asset_name() -> anyhow::Result<String> {
    let architecture = match std::env::consts::ARCH {
        "x86_64" => "x86_64",
        other => bail!("no prebuilt shared library is published for {other}"),
    };
    if std::env::consts::OS != "linux" {
        bail!("--install-so is currently supported only on Linux");
    }
    Ok(format!("libwikiquote_fetcher-{architecture}-linux.so"))
}

fn existing_library_dirs(user_dir: &Path) -> Vec<PathBuf> {
    let mut directories = vec![user_dir.to_path_buf()];
    for system_dir in ["/usr/local/lib", "/usr/lib", "/usr/lib64"] {
        let path = PathBuf::from(system_dir);
        if !directories.contains(&path) {
            directories.push(path);
        }
    }
    directories
}

#[cfg(unix)]
fn install_download(directory: &Path, contents: &[u8]) -> anyhow::Result<PathBuf> {
    use std::os::unix::fs::{PermissionsExt, symlink};

    if !contents.starts_with(b"\x7fELF") {
        bail!("downloaded release asset is not an ELF shared library");
    }

    fs::create_dir_all(directory)
        .with_context(|| format!("could not create {}", directory.display()))?;
    let versioned_name = format!("{LIBRARY_NAME}.{VERSION}");
    let destination = directory.join(&versioned_name);
    let temporary = directory.join(format!(".{versioned_name}.{}.tmp", std::process::id()));
    fs::write(&temporary, contents)
        .with_context(|| format!("could not write {}", temporary.display()))?;
    fs::set_permissions(&temporary, fs::Permissions::from_mode(0o755))?;
    fs::rename(&temporary, &destination)
        .with_context(|| format!("could not install {}", destination.display()))?;

    let link = directory.join(LIBRARY_NAME);
    let temporary_link = directory.join(format!(".{LIBRARY_NAME}.{}.tmp", std::process::id()));
    let _ = fs::remove_file(&temporary_link);
    symlink(&versioned_name, &temporary_link)?;
    fs::rename(&temporary_link, &link)
        .with_context(|| format!("could not update {}", link.display()))?;
    Ok(destination)
}

#[cfg(not(unix))]
fn install_download(_directory: &Path, _contents: &[u8]) -> anyhow::Result<PathBuf> {
    bail!("--install-so is currently supported only on Unix systems")
}

pub fn install(requested_directory: Option<PathBuf>) -> anyhow::Result<()> {
    let user_dir = user_library_dir()?;
    let scan_dirs = if let Some(directory) = &requested_directory {
        vec![directory.clone()]
    } else {
        existing_library_dirs(&user_dir)
    };

    for directory in &scan_dirs {
        if let Some((installed, path)) = installed_version(directory)
            && compare_versions(&installed, VERSION).is_some_and(|order| order.is_ge())
        {
            println!(
                "Shared library {installed} is already up to date at {}",
                path.display()
            );
            return Ok(());
        }
    }

    let using_default_directory = requested_directory.is_none();
    let target_dir = requested_directory.unwrap_or(user_dir);
    let asset = release_asset_name()?;
    let url = format!(
        "https://github.com/efebaykaraa/wikiquote-fetcher/releases/download/v{VERSION}/{asset}"
    );
    println!("Downloading shared library {VERSION} from {url}");
    let mut response = ureq::get(&url)
        .call()
        .with_context(|| format!("could not download {url}"))?;
    let contents = response.body_mut().read_to_vec()?;
    let installed = install_download(&target_dir, &contents)?;
    println!(
        "Installed shared library {VERSION} at {}",
        installed.display()
    );
    if using_default_directory {
        println!(
            "If needed, add {} to LD_LIBRARY_PATH or your dynamic linker configuration.",
            target_dir.display()
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compares_numeric_versions() {
        assert_eq!(compare_versions("1.1.0", "1.0.9"), Some(Ordering::Greater));
        assert_eq!(compare_versions("1.1", "1.1.0"), Some(Ordering::Equal));
        assert_eq!(compare_versions("2.0.0", "2.0.1"), Some(Ordering::Less));
    }

    #[cfg(unix)]
    #[test]
    fn reads_version_from_library_symlink() {
        use std::os::unix::fs::symlink;

        let directory = std::env::temp_dir().join(format!(
            "wikiquote-fetcher-version-test-{}",
            std::process::id()
        ));
        fs::create_dir_all(&directory).unwrap();
        let link = directory.join(LIBRARY_NAME);
        let _ = fs::remove_file(&link);
        symlink(format!("{LIBRARY_NAME}.1.2.3"), &link).unwrap();
        assert_eq!(
            installed_version(&directory).map(|(version, _)| version),
            Some("1.2.3".to_string())
        );
        fs::remove_file(link).unwrap();
        fs::remove_dir(directory).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn installs_versioned_library_and_updates_symlink() {
        let directory = std::env::temp_dir().join(format!(
            "wikiquote-fetcher-install-test-{}",
            std::process::id()
        ));
        fs::create_dir_all(&directory).unwrap();
        let destination = install_download(&directory, b"\x7fELFtest-library").unwrap();
        assert_eq!(
            destination.file_name().and_then(|name| name.to_str()),
            Some(&*format!("{LIBRARY_NAME}.{VERSION}"))
        );
        assert_eq!(
            installed_version(&directory).map(|(version, _)| version),
            Some(VERSION.to_string())
        );
        fs::remove_file(directory.join(LIBRARY_NAME)).unwrap();
        fs::remove_file(destination).unwrap();
        fs::remove_dir(directory).unwrap();
    }
}
