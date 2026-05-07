use std::process::ExitCode;

#[cfg(feature = "gui")]
use gamescope_gui::args::{ParseOutcome, parse_from_env};
#[cfg(feature = "gui")]
use gamescope_gui::gamescope::LaunchError;
#[cfg(all(feature = "gui", unix))]
use gamescope_gui::gamescope::exec_gamescope;
#[cfg(all(feature = "gui", not(unix)))]
use gamescope_gui::gamescope::{process_exit_code, run_gamescope};
#[cfg(feature = "gui")]
use gamescope_gui::profiles::{ProfileIdentity, ProfileStore};

#[cfg(feature = "gui")]
mod ui;

#[cfg(feature = "gui")]
fn main() -> ExitCode {
    match real_main() {
        Ok(code) => code,
        Err(err) => {
            eprintln!("{err}");
            ExitCode::from(1)
        }
    }
}

#[cfg(feature = "gui")]
fn real_main() -> Result<ExitCode, String> {
    let request = match parse_from_env().map_err(|err| err.to_string())? {
        ParseOutcome::Run(request) => request,
        ParseOutcome::Exit(code) => return Ok(exit_code_from_i32(code)),
    };

    let identity = ProfileIdentity::from_current_env(&request.game_command);
    let mut store = ProfileStore::load().map_err(|err| err.to_string())?;
    let settings = store.settings_for(&identity);

    let settings = match ui::run_settings_ui(&identity, settings, &request.game_command) {
        ui::UiOutcome::Start(settings) => settings,
        ui::UiOutcome::Cancel => return Ok(ExitCode::SUCCESS),
    };

    store.upsert(&identity, settings.clone());
    store.save().map_err(|err| err.to_string())?;

    #[cfg(unix)]
    {
        return Err(render_launch_error(exec_gamescope(
            &settings,
            &request.game_command,
        )));
    }

    #[cfg(not(unix))]
    {
        let status =
            run_gamescope(&settings, &request.game_command).map_err(render_launch_error)?;
        Ok(exit_code_from_i32(process_exit_code(status)))
    }
}

#[cfg(feature = "gui")]
fn render_launch_error(error: LaunchError) -> String {
    match error {
        LaunchError::Spawn(source) if source.kind() == std::io::ErrorKind::NotFound => {
            "failed to start gamescope: gamescope was not found in PATH".to_string()
        }
        other => other.to_string(),
    }
}

#[cfg(feature = "gui")]
fn exit_code_from_i32(code: i32) -> ExitCode {
    ExitCode::from(code.clamp(0, u8::MAX as i32) as u8)
}

#[cfg(not(feature = "gui"))]
fn main() -> ExitCode {
    eprintln!("gamescope-gui was built without the `gui` feature");
    ExitCode::from(1)
}
