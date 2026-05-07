use std::cell::RefCell;
use std::ffi::OsString;
use std::rc::Rc;

use adw::prelude::*;
use gamescope_gui::profiles::ProfileIdentity;
use gamescope_gui::settings::{Filter, GamescopeSettings, Resolution, Scaler, WindowMode};
use gtk::gdk;
use gtk::subclass::prelude::ObjectSubclassIsExt;
use gtk::{gio, glib};

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
    nested_preset: adw::ComboRow,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ResolutionPreset {
    label: &'static str,
    resolution: Resolution,
}

mod imp {
    use gtk::subclass::prelude::*;
    use gtk::{CompositeTemplate, TemplateChild, glib};

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(file = "src/ui/window.blp")]
    pub struct GamescopeContent {
        #[template_child]
        pub window_title: TemplateChild<adw::WindowTitle>,
        #[template_child]
        pub cancel_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub start_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub output_enabled: TemplateChild<adw::SwitchRow>,
        #[template_child]
        pub output_width: TemplateChild<adw::SpinRow>,
        #[template_child]
        pub output_height: TemplateChild<adw::SpinRow>,
        #[template_child]
        pub nested_enabled: TemplateChild<adw::SwitchRow>,
        #[template_child]
        pub nested_preset: TemplateChild<adw::ComboRow>,
        #[template_child]
        pub nested_width: TemplateChild<adw::SpinRow>,
        #[template_child]
        pub nested_height: TemplateChild<adw::SpinRow>,
        #[template_child]
        pub refresh_enabled: TemplateChild<adw::SwitchRow>,
        #[template_child]
        pub refresh: TemplateChild<adw::SpinRow>,
        #[template_child]
        pub fps_enabled: TemplateChild<adw::SwitchRow>,
        #[template_child]
        pub fps: TemplateChild<adw::SpinRow>,
        #[template_child]
        pub window_mode: TemplateChild<adw::ComboRow>,
        #[template_child]
        pub scaler: TemplateChild<adw::ComboRow>,
        #[template_child]
        pub filter: TemplateChild<adw::ComboRow>,
        #[template_child]
        pub hdr: TemplateChild<adw::SwitchRow>,
        #[template_child]
        pub adaptive_sync: TemplateChild<adw::SwitchRow>,
        #[template_child]
        pub integration_group: TemplateChild<adw::PreferencesGroup>,
        #[template_child]
        pub mangoapp: TemplateChild<adw::SwitchRow>,
        #[template_child]
        pub steam: TemplateChild<adw::SwitchRow>,
        #[template_child]
        pub extra_args: TemplateChild<adw::EntryRow>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for GamescopeContent {
        const NAME: &'static str = "GamescopeContent";
        type Type = super::GamescopeContent;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for GamescopeContent {}
    impl WidgetImpl for GamescopeContent {}
    impl BoxImpl for GamescopeContent {}
}

glib::wrapper! {
    pub struct GamescopeContent(ObjectSubclass<imp::GamescopeContent>)
        @extends gtk::Widget, gtk::Box,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl GamescopeContent {
    fn new() -> Self {
        glib::Object::new()
    }

    fn configure(
        &self,
        identity: &ProfileIdentity,
        settings: &GamescopeSettings,
        command_preview: &str,
    ) -> SettingsRows {
        let imp = self.imp();
        let defaults = DisplayDefaults::detect().unwrap_or_default();
        imp.window_title.set_subtitle(&identity.label);
        imp.start_button.add_css_class("suggested-action");
        imp.integration_group.set_description(Some(command_preview));

        let output_resolution = settings.output_resolution.unwrap_or(defaults.native);
        let nested_resolution = settings.nested_resolution.unwrap_or(defaults.native);

        imp.output_enabled.set_active(true);
        imp.output_enabled.set_subtitle(&format!(
            "Native monitor target: {}x{}",
            defaults.native.width, defaults.native.height
        ));
        imp.output_width.set_value(output_resolution.width as f64);
        imp.output_height.set_value(output_resolution.height as f64);
        imp.nested_enabled.set_active(true);
        imp.nested_enabled.set_subtitle(&format!(
            "Suggested from monitor: {}",
            defaults
                .presets
                .iter()
                .map(|preset| format!(
                    "{} {}x{}",
                    preset.label, preset.resolution.width, preset.resolution.height
                ))
                .collect::<Vec<_>>()
                .join(", ")
        ));
        let preset_labels = defaults
            .presets
            .iter()
            .map(|preset| {
                format!(
                    "{} - {}x{}",
                    preset.label, preset.resolution.width, preset.resolution.height
                )
            })
            .collect::<Vec<_>>();
        set_combo_model_from_strings(
            &imp.nested_preset,
            &preset_labels,
            defaults.preset_index_for(nested_resolution),
        );
        imp.nested_width.set_value(nested_resolution.width as f64);
        imp.nested_height.set_value(nested_resolution.height as f64);
        imp.refresh_enabled
            .set_active(settings.nested_refresh.is_some());
        imp.refresh
            .set_value(settings.nested_refresh.unwrap_or(60) as f64);
        imp.fps_enabled
            .set_active(settings.framerate_limit.is_some());
        imp.fps
            .set_value(settings.framerate_limit.unwrap_or(60) as f64);

        set_combo_model(
            &imp.window_mode,
            &WindowMode::LABELS,
            settings.window_mode.index(),
        );
        set_combo_model(&imp.scaler, &Scaler::LABELS, settings.scaler.index());
        set_combo_model(&imp.filter, &Filter::LABELS, settings.filter.index());

        imp.hdr.set_active(settings.hdr);
        imp.adaptive_sync.set_active(settings.adaptive_sync);
        imp.mangoapp.set_active(settings.mangoapp);
        imp.steam.set_active(settings.steam);
        imp.extra_args.set_text(&settings.extra_args);

        SettingsRows {
            output_enabled: imp.output_enabled.get(),
            output_width: imp.output_width.get(),
            output_height: imp.output_height.get(),
            nested_enabled: imp.nested_enabled.get(),
            nested_preset: imp.nested_preset.get(),
            nested_width: imp.nested_width.get(),
            nested_height: imp.nested_height.get(),
            refresh_enabled: imp.refresh_enabled.get(),
            refresh: imp.refresh.get(),
            fps_enabled: imp.fps_enabled.get(),
            fps: imp.fps.get(),
            window_mode: imp.window_mode.get(),
            scaler: imp.scaler.get(),
            filter: imp.filter.get(),
            hdr: imp.hdr.get(),
            adaptive_sync: imp.adaptive_sync.get(),
            mangoapp: imp.mangoapp.get(),
            steam: imp.steam.get(),
            extra_args: imp.extra_args.get(),
        }
    }
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

    let content = GamescopeContent::new();
    let rows = content.configure(identity, &settings, command_preview);
    connect_resolution_preset(&rows);
    window.set_content(Some(&content));

    {
        let app = app.clone();
        content.imp().cancel_button.connect_clicked(move |_| {
            app.quit();
        });
    }

    {
        let app = app.clone();
        let window = window.clone();
        content.imp().start_button.connect_clicked(move |_| {
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

fn connect_resolution_preset(rows: &SettingsRows) {
    let nested_width = rows.nested_width.clone();
    let nested_height = rows.nested_height.clone();
    rows.nested_preset.connect_selected_notify(move |row| {
        let Some(label) = selected_string(row) else {
            return;
        };
        let Some(resolution) = parse_preset_resolution(&label) else {
            return;
        };

        nested_width.set_value(resolution.width as f64);
        nested_height.set_value(resolution.height as f64);
    });
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

fn set_combo_model(row: &adw::ComboRow, labels: &[&str], selected: u32) {
    let model = gtk::StringList::new(labels);
    row.set_model(Some(&model));
    row.set_selected(selected);
}

fn set_combo_model_from_strings(row: &adw::ComboRow, labels: &[String], selected: u32) {
    let model = labels.iter().cloned().collect::<gtk::StringList>();
    row.set_model(Some(&model));
    row.set_selected(selected.min(labels.len().saturating_sub(1) as u32));
}

fn selected_string(row: &adw::ComboRow) -> Option<String> {
    row.model()?
        .item(row.selected())?
        .downcast::<gtk::StringObject>()
        .ok()
        .map(|item| item.string().to_string())
}

fn parse_preset_resolution(label: &str) -> Option<Resolution> {
    let resolution = label.rsplit_once(' ')?.1;
    let (width, height) = resolution.split_once('x')?;
    Some(Resolution {
        width: width.parse().ok()?,
        height: height.parse().ok()?,
    })
}

#[derive(Debug, Clone)]
struct DisplayDefaults {
    native: Resolution,
    presets: Vec<ResolutionPreset>,
}

impl DisplayDefaults {
    fn detect() -> Option<Self> {
        let display = gdk::Display::default()?;
        let monitors = display.monitors();
        let monitor = monitors.item(0)?.downcast::<gdk::Monitor>().ok()?;
        let geometry = monitor.geometry();
        let native = Resolution {
            width: geometry.width().try_into().ok()?,
            height: geometry.height().try_into().ok()?,
        };
        Some(Self::new(native))
    }

    fn new(native: Resolution) -> Self {
        let mut presets = Vec::new();
        push_unique_preset(&mut presets, "Native", native);
        push_unique_preset(&mut presets, "1.5x scale", scaled_resolution(native, 1.5));
        push_unique_preset(&mut presets, "2x scale", scaled_resolution(native, 2.0));

        Self { native, presets }
    }

    fn preset_index_for(&self, resolution: Resolution) -> u32 {
        self.presets
            .iter()
            .position(|preset| preset.resolution == resolution)
            .unwrap_or_default() as u32
    }
}

impl Default for DisplayDefaults {
    fn default() -> Self {
        Self::new(Resolution {
            width: 1920,
            height: 1080,
        })
    }
}

fn push_unique_preset(
    presets: &mut Vec<ResolutionPreset>,
    label: &'static str,
    resolution: Resolution,
) {
    if resolution.width < 320 || resolution.height < 240 {
        return;
    }
    if presets.iter().any(|preset| preset.resolution == resolution) {
        return;
    }
    presets.push(ResolutionPreset { label, resolution });
}

fn scaled_resolution(native: Resolution, scale: f64) -> Resolution {
    Resolution {
        width: round_to_even((native.width as f64 / scale).round() as u32),
        height: round_to_even((native.height as f64 / scale).round() as u32),
    }
}

fn round_to_even(value: u32) -> u32 {
    value - (value % 2)
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
