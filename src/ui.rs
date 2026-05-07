use std::cell::RefCell;
use std::ffi::OsString;
use std::rc::Rc;

use adw::prelude::*;
use gamescope_gui::profiles::ProfileIdentity;
use gamescope_gui::settings::{Filter, GamescopeSettings, Resolution, Scaler, WindowMode};
use gtk::gio;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UiOutcome {
    Start(GamescopeSettings),
    Cancel,
}

pub fn run_settings_ui(
    identity: &ProfileIdentity,
    initial_settings: GamescopeSettings,
    game_command: &[OsString],
) -> UiOutcome {
    let app = adw::Application::new(
        Some("io.github.andriishafar.GamescopeGui"),
        gio::ApplicationFlags::NON_UNIQUE,
    );
    let outcome = Rc::new(RefCell::new(UiOutcome::Cancel));
    let identity = identity.clone();
    let command_preview = render_command_preview(game_command);

    {
        let outcome = Rc::clone(&outcome);
        app.connect_activate(move |app| {
            build_window(
                app,
                &identity,
                initial_settings.clone(),
                &command_preview,
                Rc::clone(&outcome),
            );
        });
    }

    app.run_with_args(&["gamescope-gui"]);
    let final_outcome = outcome.borrow().clone();
    final_outcome
}

#[derive(Clone)]
struct SettingsRows {
    output_enabled: adw::SwitchRow,
    output_width: adw::SpinRow,
    output_height: adw::SpinRow,
    nested_enabled: adw::SwitchRow,
    nested_width: adw::SpinRow,
    nested_height: adw::SpinRow,
    refresh_enabled: adw::SwitchRow,
    refresh: adw::SpinRow,
    fps_enabled: adw::SwitchRow,
    fps: adw::SpinRow,
    window_mode: adw::ComboRow,
    scaler: adw::ComboRow,
    filter: adw::ComboRow,
    hdr: adw::SwitchRow,
    adaptive_sync: adw::SwitchRow,
    mangoapp: adw::SwitchRow,
    steam: adw::SwitchRow,
    extra_args: adw::EntryRow,
}

fn build_window(
    app: &adw::Application,
    identity: &ProfileIdentity,
    settings: GamescopeSettings,
    command_preview: &str,
    outcome: Rc<RefCell<UiOutcome>>,
) {
    let window = adw::ApplicationWindow::builder()
        .application(app)
        .title("Gamescope")
        .default_width(560)
        .default_height(720)
        .build();

    let toolbar_view = adw::ToolbarView::new();
    let header = adw::HeaderBar::new();
    let title = adw::WindowTitle::new("Gamescope", &identity.label);
    header.set_title_widget(Some(&title));

    let cancel_button = gtk::Button::with_label("Cancel");
    let start_button = gtk::Button::with_label("Start");
    start_button.add_css_class("suggested-action");
    header.pack_start(&cancel_button);
    header.pack_end(&start_button);
    toolbar_view.add_top_bar(&header);

    let page = adw::PreferencesPage::new();
    let rows = build_preferences(&page, &settings, command_preview);

    let scrolled = gtk::ScrolledWindow::builder()
        .hscrollbar_policy(gtk::PolicyType::Never)
        .child(&page)
        .build();
    toolbar_view.set_content(Some(&scrolled));
    window.set_content(Some(&toolbar_view));

    {
        let app = app.clone();
        cancel_button.connect_clicked(move |_| {
            app.quit();
        });
    }

    {
        let app = app.clone();
        let window = window.clone();
        start_button.connect_clicked(move |_| {
            let settings = rows.to_settings();
            if let Err(err) = settings.to_gamescope_args() {
                show_error(&window, "Invalid Gamescope settings", &err.to_string());
                return;
            }

            if let Err(err) = which::which("gamescope") {
                show_error(
                    &window,
                    "Gamescope was not found",
                    &format!("Install gamescope or make sure it is available in PATH. {err}"),
                );
                return;
            }

            *outcome.borrow_mut() = UiOutcome::Start(settings);
            app.quit();
        });
    }

    window.present();
}

fn build_preferences(
    page: &adw::PreferencesPage,
    settings: &GamescopeSettings,
    command_preview: &str,
) -> SettingsRows {
    let display_group = adw::PreferencesGroup::builder()
        .title("Display")
        .description("Configure the Gamescope window and game-side resolution.")
        .build();

    let output_enabled = switch_row(
        "Custom output resolution",
        settings.output_resolution.is_some(),
    );
    let output_width = spin_row(
        "Output width",
        settings
            .output_resolution
            .map(|resolution| resolution.width)
            .unwrap_or(1920),
        320.0,
        7680.0,
        1.0,
    );
    let output_height = spin_row(
        "Output height",
        settings
            .output_resolution
            .map(|resolution| resolution.height)
            .unwrap_or(1080),
        240.0,
        4320.0,
        1.0,
    );
    let nested_enabled = switch_row(
        "Custom game resolution",
        settings.nested_resolution.is_some(),
    );
    let nested_width = spin_row(
        "Game width",
        settings
            .nested_resolution
            .map(|resolution| resolution.width)
            .unwrap_or(1280),
        320.0,
        7680.0,
        1.0,
    );
    let nested_height = spin_row(
        "Game height",
        settings
            .nested_resolution
            .map(|resolution| resolution.height)
            .unwrap_or(720),
        240.0,
        4320.0,
        1.0,
    );

    display_group.add(&output_enabled);
    display_group.add(&output_width);
    display_group.add(&output_height);
    display_group.add(&nested_enabled);
    display_group.add(&nested_width);
    display_group.add(&nested_height);
    page.add(&display_group);

    let performance_group = adw::PreferencesGroup::builder()
        .title("Performance")
        .build();
    let refresh_enabled = switch_row("Limit refresh rate", settings.nested_refresh.is_some());
    let refresh = spin_row(
        "Refresh rate",
        settings.nested_refresh.unwrap_or(60),
        1.0,
        1000.0,
        1.0,
    );
    let fps_enabled = switch_row("Limit FPS", settings.framerate_limit.is_some());
    let fps = spin_row(
        "FPS limit",
        settings.framerate_limit.unwrap_or(60),
        1.0,
        1000.0,
        1.0,
    );
    performance_group.add(&refresh_enabled);
    performance_group.add(&refresh);
    performance_group.add(&fps_enabled);
    performance_group.add(&fps);
    page.add(&performance_group);

    let rendering_group = adw::PreferencesGroup::builder().title("Rendering").build();
    let window_mode = combo_row(
        "Window mode",
        &WindowMode::LABELS,
        settings.window_mode.index(),
    );
    let scaler = combo_row("Scaler", &Scaler::LABELS, settings.scaler.index());
    let filter = combo_row("Filter", &Filter::LABELS, settings.filter.index());
    let hdr = switch_row("HDR", settings.hdr);
    let adaptive_sync = switch_row("Variable refresh rate", settings.adaptive_sync);
    rendering_group.add(&window_mode);
    rendering_group.add(&scaler);
    rendering_group.add(&filter);
    rendering_group.add(&hdr);
    rendering_group.add(&adaptive_sync);
    page.add(&rendering_group);

    let integration_group = adw::PreferencesGroup::builder()
        .title("Integration")
        .description(command_preview)
        .build();
    let mangoapp = switch_row("MangoHud overlay", settings.mangoapp);
    let steam = adw::SwitchRow::builder()
        .title("Gamescope Steam integration mode")
        .subtitle("Advanced; intended for running Steam itself inside Gamescope, not normal per-game Steam launch options.")
        .build();
    steam.set_active(settings.steam);
    let extra_args = adw::EntryRow::new();
    extra_args.set_title("Extra Gamescope arguments");
    extra_args.set_text(&settings.extra_args);
    integration_group.add(&mangoapp);
    integration_group.add(&steam);
    integration_group.add(&extra_args);
    page.add(&integration_group);

    SettingsRows {
        output_enabled,
        output_width,
        output_height,
        nested_enabled,
        nested_width,
        nested_height,
        refresh_enabled,
        refresh,
        fps_enabled,
        fps,
        window_mode,
        scaler,
        filter,
        hdr,
        adaptive_sync,
        mangoapp,
        steam,
        extra_args,
    }
}

impl SettingsRows {
    fn to_settings(&self) -> GamescopeSettings {
        GamescopeSettings {
            output_resolution: self.output_enabled.is_active().then(|| Resolution {
                width: self.output_width.value() as u32,
                height: self.output_height.value() as u32,
            }),
            nested_resolution: self.nested_enabled.is_active().then(|| Resolution {
                width: self.nested_width.value() as u32,
                height: self.nested_height.value() as u32,
            }),
            nested_refresh: self
                .refresh_enabled
                .is_active()
                .then(|| self.refresh.value() as u32),
            framerate_limit: self
                .fps_enabled
                .is_active()
                .then(|| self.fps.value() as u32),
            window_mode: WindowMode::from_index(self.window_mode.selected()),
            scaler: Scaler::from_index(self.scaler.selected()),
            filter: Filter::from_index(self.filter.selected()),
            hdr: self.hdr.is_active(),
            adaptive_sync: self.adaptive_sync.is_active(),
            mangoapp: self.mangoapp.is_active(),
            steam: self.steam.is_active(),
            extra_args: self.extra_args.text().to_string(),
        }
    }
}

fn switch_row(title: &str, active: bool) -> adw::SwitchRow {
    let row = adw::SwitchRow::builder().title(title).build();
    row.set_active(active);
    row
}

fn spin_row(title: &str, value: u32, min: f64, max: f64, step: f64) -> adw::SpinRow {
    let row = adw::SpinRow::with_range(min, max, step);
    row.set_title(title);
    row.set_value(value as f64);
    row
}

fn combo_row(title: &str, labels: &[&str], selected: u32) -> adw::ComboRow {
    let model = gtk::StringList::new(labels);
    adw::ComboRow::builder()
        .title(title)
        .model(&model)
        .selected(selected)
        .build()
}

fn show_error(parent: &adw::ApplicationWindow, heading: &str, body: &str) {
    let dialog = adw::MessageDialog::new(Some(parent), Some(heading), Some(body));
    dialog.add_responses(&[("ok", "OK")]);
    dialog.present();
}

fn render_command_preview(command: &[OsString]) -> String {
    command
        .iter()
        .map(|arg| arg.to_string_lossy())
        .collect::<Vec<_>>()
        .join(" ")
}
