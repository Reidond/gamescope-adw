use std::ffi::OsString;
use std::io;
use std::process::{Command, ExitStatus};

#[cfg(unix)]
use std::os::unix::process::{CommandExt, ExitStatusExt};

use thiserror::Error;

use crate::settings::{GamescopeSettings, SettingsError};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GamescopeCommand {
    pub program: OsString,
    pub args: Vec<OsString>,
}

#[derive(Debug, Error)]
pub enum LaunchError {
    #[error("{0}")]
    Settings(#[from] SettingsError),
    #[error("failed to start gamescope: {0}")]
    Spawn(io::Error),
    #[error("failed while waiting for gamescope: {0}")]
    Wait(io::Error),
}

pub fn build_gamescope_command(
    settings: &GamescopeSettings,
    game_command: &[OsString],
) -> Result<GamescopeCommand, SettingsError> {
    let mut args = settings.to_gamescope_args()?;
    args.push("--".into());
    args.extend(game_command.iter().cloned());

    Ok(GamescopeCommand {
        program: "gamescope".into(),
        args,
    })
}

pub fn run_gamescope(
    settings: &GamescopeSettings,
    game_command: &[OsString],
) -> Result<ExitStatus, LaunchError> {
    let command = build_gamescope_command(settings, game_command)?;
    let mut child = Command::new(&command.program)
        .args(&command.args)
        .spawn()
        .map_err(LaunchError::Spawn)?;

    child.wait().map_err(LaunchError::Wait)
}

#[cfg(unix)]
pub fn exec_gamescope(settings: &GamescopeSettings, game_command: &[OsString]) -> LaunchError {
    let command = match build_gamescope_command(settings, game_command) {
        Ok(command) => command,
        Err(error) => return LaunchError::Settings(error),
    };

    let error = Command::new(&command.program).args(&command.args).exec();
    LaunchError::Spawn(error)
}

pub fn process_exit_code(status: ExitStatus) -> i32 {
    if let Some(code) = status.code() {
        return code;
    }

    #[cfg(unix)]
    if let Some(signal) = status.signal() {
        return 128 + signal;
    }

    1
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings::{GamescopeSettings, Resolution, WindowMode};

    fn os_args(args: &[&str]) -> Vec<OsString> {
        args.iter().map(OsString::from).collect()
    }

    #[test]
    fn inserts_separator_before_game_command() {
        let settings = GamescopeSettings {
            output_resolution: Some(Resolution {
                width: 1920,
                height: 1080,
            }),
            window_mode: WindowMode::Borderless,
            steam: true,
            ..Default::default()
        };

        let command = build_gamescope_command(&settings, &os_args(&["game", "-safe"]))
            .expect("valid command");
        let rendered: Vec<String> = command
            .args
            .iter()
            .map(|arg| arg.to_string_lossy().into_owned())
            .collect();

        assert_eq!(
            rendered,
            vec![
                "-W", "1920", "-H", "1080", "-b", "--steam", "--", "game", "-safe"
            ]
        );
    }
}
