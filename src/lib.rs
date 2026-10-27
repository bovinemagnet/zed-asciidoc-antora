use std::fs;

use zed_extension_api as zed;
use zed_extension_api::settings::{CommandSettings, LspSettings};

const SERVER_BINARY: &str = "adoc-ls";
const SERVER_REPOSITORY: &str = "bovinemagnet/AdocFoundry";
const STDIO_ARG: &str = "--stdio";

struct AsciiDocExtension {
    cached_binary_path: Option<String>,
}

fn release_version(tag: &str) -> &str {
    tag.strip_prefix('v').unwrap_or(tag)
}

/// Names the AdocFoundry release asset that carries `adoc-ls` for the platform.
fn release_asset(
    tag: &str,
    os: zed::Os,
    arch: zed::Architecture,
) -> zed::Result<(String, zed::DownloadedFileType)> {
    use zed::{Architecture, DownloadedFileType, Os};

    let (target, extension, file_type) = match (os, arch) {
        (Os::Linux, Architecture::X8664) => ("linux-x86_64", "tar.gz", DownloadedFileType::GzipTar),
        (Os::Linux, Architecture::Aarch64) => {
            ("linux-aarch64", "tar.gz", DownloadedFileType::GzipTar)
        }
        (Os::Mac, Architecture::X8664 | Architecture::Aarch64) => {
            ("macos-universal", "tar.gz", DownloadedFileType::GzipTar)
        }
        (Os::Windows, Architecture::X8664) => ("windows-x64", "zip", DownloadedFileType::Zip),
        (Os::Windows, Architecture::Aarch64) => ("windows-arm64", "zip", DownloadedFileType::Zip),
        (os, arch) => {
            return Err(format!(
                "no prebuilt {SERVER_BINARY} is published for {os:?} {arch:?}; install it with `cargo install --git https://github.com/bovinemagnet/AdocFoundry adoc-ls` and put it on PATH"
            ))
        }
    };

    Ok((
        format!(
            "{SERVER_BINARY}-{}-{target}.{extension}",
            release_version(tag)
        ),
        file_type,
    ))
}

fn binary_name(os: zed::Os) -> String {
    match os {
        zed::Os::Windows => format!("{SERVER_BINARY}.exe"),
        zed::Os::Mac | zed::Os::Linux => SERVER_BINARY.to_owned(),
    }
}

fn version_dir(tag: &str) -> String {
    format!("{SERVER_BINARY}-{}", release_version(tag))
}

fn is_file(path: &str) -> bool {
    fs::metadata(path).is_ok_and(|metadata| metadata.is_file())
}

/// Picks the download directory with the highest release version.
fn newest_version_dir(names: impl IntoIterator<Item = String>) -> Option<String> {
    names
        .into_iter()
        .filter_map(|name| {
            let version = name
                .strip_prefix(&format!("{SERVER_BINARY}-"))?
                .split('.')
                .map(|part| part.parse::<u64>().ok())
                .collect::<Option<Vec<_>>>()?;
            Some((version, name))
        })
        .max()
        .map(|(_, name)| name)
}

/// Returns the binary from the newest earlier download, if one is on disk.
fn previously_downloaded_binary(os: zed::Os) -> Option<String> {
    let names = fs::read_dir(".")
        .ok()?
        .flatten()
        .map(|entry| entry.file_name().to_string_lossy().into_owned());
    let binary_path = format!("{}/{}", newest_version_dir(names)?, binary_name(os));
    is_file(&binary_path).then_some(binary_path)
}

impl AsciiDocExtension {
    /// Returns `adoc-ls` from the latest AdocFoundry release, falling back to an
    /// earlier download when the release cannot be fetched.
    fn downloaded_binary_path(
        &mut self,
        language_server_id: &zed::LanguageServerId,
    ) -> zed::Result<String> {
        if let Some(path) = &self.cached_binary_path {
            if is_file(path) {
                return Ok(path.clone());
            }
        }

        zed::set_language_server_installation_status(
            language_server_id,
            &zed::LanguageServerInstallationStatus::CheckingForUpdate,
        );
        let (os, arch) = zed::current_platform();
        let binary_path = match update_binary(language_server_id, os, arch) {
            Ok(binary_path) => binary_path,
            Err(error) => match previously_downloaded_binary(os) {
                Some(binary_path) => binary_path,
                None => {
                    zed::set_language_server_installation_status(
                        language_server_id,
                        &zed::LanguageServerInstallationStatus::Failed(error.clone()),
                    );
                    return Err(error);
                }
            },
        };

        zed::set_language_server_installation_status(
            language_server_id,
            &zed::LanguageServerInstallationStatus::None,
        );
        self.cached_binary_path = Some(binary_path.clone());
        Ok(binary_path)
    }
}

/// Downloads `adoc-ls` from the latest AdocFoundry release into the extension's
/// working directory, removing binaries left by older releases.
fn update_binary(
    language_server_id: &zed::LanguageServerId,
    os: zed::Os,
    arch: zed::Architecture,
) -> zed::Result<String> {
    let release = zed::latest_github_release(
        SERVER_REPOSITORY,
        zed::GithubReleaseOptions {
            require_assets: true,
            pre_release: false,
        },
    )?;

    let (asset_name, file_type) = release_asset(&release.version, os, arch)?;
    let asset = release
        .assets
        .iter()
        .find(|asset| asset.name == asset_name)
        .ok_or_else(|| format!("release {} has no asset {asset_name}", release.version))?;

    let version_dir = version_dir(&release.version);
    let binary_path = format!("{version_dir}/{}", binary_name(os));

    if !is_file(&binary_path) {
        zed::set_language_server_installation_status(
            language_server_id,
            &zed::LanguageServerInstallationStatus::Downloading,
        );
        if let Err(error) = zed::download_file(&asset.download_url, &version_dir, file_type)
            .and_then(|()| zed::make_file_executable(&binary_path))
        {
            // Remove a partial download so the fallback never picks it up.
            fs::remove_dir_all(&version_dir).ok();
            return Err(format!("failed to install {asset_name}: {error}"));
        }

        let entries =
            fs::read_dir(".").map_err(|error| format!("failed to list downloads: {error}"))?;
        for entry in entries.flatten() {
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if name.starts_with(&format!("{SERVER_BINARY}-")) && name != version_dir {
                fs::remove_dir_all(entry.path()).ok();
            }
        }
    }

    Ok(binary_path)
}

/// Builds the command Zed runs, taking arguments and extra environment from the
/// `lsp.adoc-ls.binary` settings when they are given.
fn server_command(
    binary: String,
    settings: Option<&CommandSettings>,
    mut env: Vec<(String, String)>,
) -> zed::Command {
    let args = settings
        .and_then(|settings| settings.arguments.clone())
        .unwrap_or_else(|| vec![STDIO_ARG.to_owned()]);

    if let Some(extra) = settings.and_then(|settings| settings.env.as_ref()) {
        env.retain(|(key, _)| !extra.contains_key(key));
        env.extend(
            extra
                .iter()
                .map(|(key, value)| (key.clone(), value.clone())),
        );
    }

    zed::Command {
        command: binary,
        args,
        env,
    }
}

impl zed::Extension for AsciiDocExtension {
    fn new() -> Self {
        Self {
            cached_binary_path: None,
        }
    }

    fn language_server_command(
        &mut self,
        language_server_id: &zed::LanguageServerId,
        worktree: &zed::Worktree,
    ) -> zed::Result<zed::Command> {
        let settings = LspSettings::for_worktree(SERVER_BINARY, worktree)
            .ok()
            .and_then(|settings| settings.binary);

        let binary = match settings.as_ref().and_then(|settings| settings.path.clone()) {
            Some(path) => path,
            None => match worktree.which(SERVER_BINARY) {
                Some(binary) => binary,
                None => self.downloaded_binary_path(language_server_id)?,
            },
        };

        Ok(server_command(
            binary,
            settings.as_ref(),
            worktree.shell_env(),
        ))
    }
}

zed::register_extension!(AsciiDocExtension);

#[cfg(test)]
mod tests {
    use super::*;

    fn binary_settings(
        arguments: Option<&[&str]>,
        env: Option<&[(&str, &str)]>,
    ) -> CommandSettings {
        CommandSettings {
            path: None,
            arguments: arguments.map(|arguments| arguments.iter().map(|a| a.to_string()).collect()),
            env: env.map(|env| {
                env.iter()
                    .map(|(key, value)| (key.to_string(), value.to_string()))
                    .collect()
            }),
        }
    }

    #[test]
    fn server_command_requests_stdio_transport() {
        let command = server_command("/usr/local/bin/adoc-ls".to_owned(), None, Vec::new());

        assert_eq!(command.args, vec!["--stdio".to_owned()]);
    }

    #[test]
    fn server_command_uses_the_resolved_binary_path() {
        let command = server_command("/opt/adoc-ls".to_owned(), None, Vec::new());

        assert_eq!(command.command, "/opt/adoc-ls");
    }

    #[test]
    fn server_command_forwards_the_shell_environment() {
        let env = vec![("PATH".to_owned(), "/usr/bin".to_owned())];

        let command = server_command("adoc-ls".to_owned(), None, env.clone());

        assert_eq!(command.env, env);
    }

    #[test]
    fn server_command_keeps_stdio_when_the_settings_give_no_arguments() {
        let settings = binary_settings(None, None);

        let command = server_command("adoc-ls".to_owned(), Some(&settings), Vec::new());

        assert_eq!(command.args, vec!["--stdio".to_owned()]);
    }

    #[test]
    fn server_command_takes_its_arguments_from_the_settings() {
        let settings = binary_settings(Some(&["--stdio", "--log", "debug"]), None);

        let command = server_command("adoc-ls".to_owned(), Some(&settings), Vec::new());

        assert_eq!(command.args, vec!["--stdio", "--log", "debug"]);
    }

    #[test]
    fn server_command_adds_the_settings_environment_over_the_shell_environment() {
        let shell_env = vec![
            ("PATH".to_owned(), "/usr/bin".to_owned()),
            ("RUST_LOG".to_owned(), "warn".to_owned()),
        ];
        let settings = binary_settings(None, Some(&[("RUST_LOG", "debug"), ("EXTRA", "1")]));

        let mut env = server_command("adoc-ls".to_owned(), Some(&settings), shell_env).env;
        env.sort();

        assert_eq!(
            env,
            vec![
                ("EXTRA".to_owned(), "1".to_owned()),
                ("PATH".to_owned(), "/usr/bin".to_owned()),
                ("RUST_LOG".to_owned(), "debug".to_owned()),
            ]
        );
    }

    #[test]
    fn release_asset_selects_the_linux_tarball_for_the_architecture() {
        let (name, file_type) =
            release_asset("v0.3.0", zed::Os::Linux, zed::Architecture::X8664).unwrap();
        assert_eq!(name, "adoc-ls-0.3.0-linux-x86_64.tar.gz");
        assert_eq!(file_type, zed::DownloadedFileType::GzipTar);

        let (name, _) =
            release_asset("v0.3.0", zed::Os::Linux, zed::Architecture::Aarch64).unwrap();
        assert_eq!(name, "adoc-ls-0.3.0-linux-aarch64.tar.gz");
    }

    #[test]
    fn release_asset_selects_the_universal_macos_tarball_on_both_architectures() {
        for arch in [zed::Architecture::Aarch64, zed::Architecture::X8664] {
            let (name, file_type) = release_asset("v0.3.0", zed::Os::Mac, arch).unwrap();
            assert_eq!(name, "adoc-ls-0.3.0-macos-universal.tar.gz");
            assert_eq!(file_type, zed::DownloadedFileType::GzipTar);
        }
    }

    #[test]
    fn release_asset_selects_the_windows_zip_for_the_architecture() {
        let (name, file_type) =
            release_asset("v0.3.0", zed::Os::Windows, zed::Architecture::X8664).unwrap();
        assert_eq!(name, "adoc-ls-0.3.0-windows-x64.zip");
        assert_eq!(file_type, zed::DownloadedFileType::Zip);

        let (name, _) =
            release_asset("v0.3.0", zed::Os::Windows, zed::Architecture::Aarch64).unwrap();
        assert_eq!(name, "adoc-ls-0.3.0-windows-arm64.zip");
    }

    #[test]
    fn release_asset_accepts_a_tag_without_the_v_prefix() {
        let (name, _) = release_asset("0.3.0", zed::Os::Linux, zed::Architecture::X8664).unwrap();
        assert_eq!(name, "adoc-ls-0.3.0-linux-x86_64.tar.gz");
    }

    #[test]
    fn release_asset_rejects_32_bit_x86_and_points_at_a_source_install() {
        let message = release_asset("v0.3.0", zed::Os::Linux, zed::Architecture::X86).unwrap_err();

        assert!(message.contains(SERVER_BINARY), "{message}");
        assert!(
            message.contains(
                "cargo install --git https://github.com/bovinemagnet/AdocFoundry adoc-ls"
            ),
            "{message}"
        );
    }

    #[test]
    fn binary_name_adds_the_windows_executable_suffix() {
        assert_eq!(binary_name(zed::Os::Windows), "adoc-ls.exe");
        assert_eq!(binary_name(zed::Os::Linux), "adoc-ls");
        assert_eq!(binary_name(zed::Os::Mac), "adoc-ls");
    }

    #[test]
    fn version_dir_names_the_download_by_release_version() {
        assert_eq!(version_dir("v0.3.0"), "adoc-ls-0.3.0");
    }

    #[test]
    fn newest_version_dir_compares_versions_numerically() {
        let names = ["adoc-ls-0.2.1", "adoc-ls-0.10.0", "adoc-ls-0.9.3"].map(String::from);

        assert_eq!(newest_version_dir(names), Some("adoc-ls-0.10.0".to_owned()));
    }

    #[test]
    fn newest_version_dir_ignores_names_that_are_not_downloads() {
        let names = ["other", "adoc-ls-latest", "adoc-ls-0.3.0", "adoc-ls"].map(String::from);

        assert_eq!(newest_version_dir(names), Some("adoc-ls-0.3.0".to_owned()));
        assert_eq!(newest_version_dir(["other".to_owned()]), None);
    }
}
