use std::ffi::OsString;
use std::io::{self, Write};

use clap::{ArgAction, Command};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LaunchRequest {
    pub game_command: Vec<OsString>,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ArgsError {
    #[error("missing `--`; use `gamescope-gui -- %command%` in Steam launch options")]
    MissingSeparator,
    #[error("missing game command after `--`; use `gamescope-gui -- %command%`")]
    MissingGameCommand,
    #[error("unsupported wrapper argument `{0}` before `--`")]
    UnsupportedArgument(String),
    #[error("failed to render help: {0}")]
    Help(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseOutcome {
    Run(LaunchRequest),
    Exit(i32),
}

pub fn parse_from_env() -> Result<ParseOutcome, ArgsError> {
    parse_args(std::env::args_os(), &mut io::stdout(), &mut io::stderr())
}

pub fn parse_args<I, W, E>(
    args: I,
    stdout: &mut W,
    stderr: &mut E,
) -> Result<ParseOutcome, ArgsError>
where
    I: IntoIterator<Item = OsString>,
    W: Write,
    E: Write,
{
    let mut iter = args.into_iter();
    let program = iter
        .next()
        .unwrap_or_else(|| OsString::from("gamescope-gui"));
    let rest: Vec<OsString> = iter.collect();

    if rest.iter().any(|arg| arg == "-h" || arg == "--help") {
        wrapper_command()
            .bin_name(program.to_string_lossy().as_ref())
            .write_long_help(stdout)
            .map_err(|err| ArgsError::Help(err.to_string()))?;
        writeln!(stdout).map_err(|err| ArgsError::Help(err.to_string()))?;
        return Ok(ParseOutcome::Exit(0));
    }

    if rest.iter().any(|arg| arg == "-V" || arg == "--version") {
        writeln!(
            stdout,
            "{} {}",
            env!("CARGO_PKG_NAME"),
            env!("CARGO_PKG_VERSION")
        )
        .map_err(|err| ArgsError::Help(err.to_string()))?;
        return Ok(ParseOutcome::Exit(0));
    }

    let Some(separator_index) = rest.iter().position(|arg| arg == "--") else {
        writeln!(stderr, "{}", ArgsError::MissingSeparator).ok();
        return Err(ArgsError::MissingSeparator);
    };

    if let Some(arg) = rest[..separator_index].first() {
        let rendered = arg.to_string_lossy().into_owned();
        writeln!(
            stderr,
            "{}",
            ArgsError::UnsupportedArgument(rendered.clone())
        )
        .ok();
        return Err(ArgsError::UnsupportedArgument(rendered));
    }

    let game_command = rest[separator_index + 1..].to_vec();
    if game_command.is_empty() {
        writeln!(stderr, "{}", ArgsError::MissingGameCommand).ok();
        return Err(ArgsError::MissingGameCommand);
    }

    Ok(ParseOutcome::Run(LaunchRequest { game_command }))
}

fn wrapper_command() -> Command {
    Command::new("gamescope-gui")
        .version(env!("CARGO_PKG_VERSION"))
        .about("Configure Gamescope per Steam game before launching it")
        .disable_help_flag(true)
        .disable_version_flag(true)
        .arg(
            clap::Arg::new("help")
                .short('h')
                .long("help")
                .action(ArgAction::Help)
                .help("Print help"),
        )
        .after_help(
            "Steam launch option:\n  gamescope-gui -- %command%\n\n\
             Everything after `--` is passed unchanged as the game command.",
        )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn os_args(args: &[&str]) -> Vec<OsString> {
        args.iter().map(OsString::from).collect()
    }

    #[test]
    fn parses_command_after_separator() {
        let mut out = Vec::new();
        let mut err = Vec::new();
        let result = parse_args(
            os_args(&["gamescope-gui", "--", "game", "-arg", "value"]),
            &mut out,
            &mut err,
        )
        .unwrap();

        assert_eq!(
            result,
            ParseOutcome::Run(LaunchRequest {
                game_command: os_args(&["game", "-arg", "value"])
            })
        );
    }

    #[test]
    fn rejects_missing_separator() {
        let mut out = Vec::new();
        let mut err = Vec::new();
        let result = parse_args(os_args(&["gamescope-gui", "game"]), &mut out, &mut err);

        assert_eq!(result, Err(ArgsError::MissingSeparator));
    }

    #[test]
    fn rejects_wrapper_args_before_separator() {
        let mut out = Vec::new();
        let mut err = Vec::new();
        let result = parse_args(
            os_args(&["gamescope-gui", "--fullscreen", "--", "game"]),
            &mut out,
            &mut err,
        );

        assert_eq!(
            result,
            Err(ArgsError::UnsupportedArgument("--fullscreen".to_string()))
        );
    }
}
