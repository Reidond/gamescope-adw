use std::cell::RefCell;
use std::ffi::OsString;
use std::rc::Rc;

use adw::prelude::*;
use gamescope_gui::display::DisplayDefaults;
use gamescope_gui::profiles::ProfileIdentity;
use gamescope_gui::settings::{Filter, GamescopeSettings, Resolution, Scaler, WindowMode};
use gtk::subclass::prelude::ObjectSubclassIsExt;
use gtk::{gdk, gio, glib};

const LAUNCH_BANNER_CSS: &str = r#"
banner.launch-banner button.text-button {
    background-color: @accent_bg_color;
    color: @accent_fg_color;
    font-weight: bold;
}
banner.launch-banner button.text-button:hover {
    background-image: image(alpha(currentColor, 0.05));
}
banner.launch-banner button.text-button:active {
    background-image: image(alpha(currentColor, 0.1));
}
"#;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UiOutcome {
    Start {
        settings: GamescopeSettings,
        display_defaults: DisplayDefaults,
    },
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
    outcome.borrow().clone()
}

#[derive(Clone)]
struct SettingsRows {
    output_resolution_row: adw::ExpanderRow,
    output_width: adw::SpinRow,
    output_height: adw::SpinRow,
    nested_resolution_row: adw::ExpanderRow,
    nested_preset: adw::ComboRow,
    nested_width: adw::SpinRow,
    nested_height: adw::SpinRow,
    refresh_row: adw::ExpanderRow,
    refresh: adw::SpinRow,
    fps_row: adw::ExpanderRow,
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

mod imp {
    use gtk::subclass::prelude::*;
    use gtk::{CompositeTemplate, TemplateChild, glib};

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(file = "src/ui/window.blp")]
    pub struct GamescopeContent {
        #[template_child]
        pub window_title: TemplateChild<adw::WindowTitle>,
        #[template_child]
        pub command_banner: TemplateChild<adw::Banner>,
        #[template_child]
        pub menu_button: TemplateChild<gtk::MenuButton>,
        #[template_child]
        pub output_resolution_row: TemplateChild<adw::ExpanderRow>,
        #[template_child]
        pub output_width: TemplateChild<adw::SpinRow>,
        #[template_child]
        pub output_height: TemplateChild<adw::SpinRow>,
        #[template_child]
        pub nested_resolution_row: TemplateChild<adw::ExpanderRow>,
        #[template_child]
        pub nested_preset: TemplateChild<adw::ComboRow>,
        #[template_child]
        pub nested_width: TemplateChild<adw::SpinRow>,
        #[template_child]
        pub nested_height: TemplateChild<adw::SpinRow>,
        #[template_child]
        pub refresh_row: TemplateChild<adw::ExpanderRow>,
        #[template_child]
        pub refresh: TemplateChild<adw::SpinRow>,
        #[template_child]
        pub fps_row: TemplateChild<adw::ExpanderRow>,
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

    fn rows(&self) -> SettingsRows {
        let imp = self.imp();
        SettingsRows {
            output_resolution_row: imp.output_resolution_row.get(),
            output_width: imp.output_width.get(),
            output_height: imp.output_height.get(),
            nested_resolution_row: imp.nested_resolution_row.get(),
            nested_preset: imp.nested_preset.get(),
            nested_width: imp.nested_width.get(),
            nested_height: imp.nested_height.get(),
            refresh_row: imp.refresh_row.get(),
            refresh: imp.refresh.get(),
            fps_row: imp.fps_row.get(),
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

    fn configure(
        &self,
        identity: &ProfileIdentity,
        settings: &GamescopeSettings,
        command_preview: &str,
        defaults: &DisplayDefaults,
    ) {
        let imp = self.imp();
        imp.window_title.set_title(&identity.label);
        imp.window_title.set_subtitle("Gamescope settings");

        imp.command_banner
            .set_title(&format!("Launching: {}", command_preview));
        imp.command_banner.set_tooltip_text(Some(command_preview));

        imp.output_resolution_row.set_subtitle(&format!(
            "Defaults to native monitor target: {}×{}",
            defaults.native.width, defaults.native.height
        ));

        let preset_labels: Vec<String> = defaults
            .presets
            .iter()
            .map(|preset| {
                format!(
                    "{} — {}×{}",
                    preset.label, preset.resolution.width, preset.resolution.height
                )
            })
            .collect();
        let preset_model = preset_labels.iter().cloned().collect::<gtk::StringList>();
        imp.nested_preset.set_model(Some(&preset_model));

        set_combo_model(
            &imp.window_mode,
            &WindowMode::LABELS,
            settings.window_mode.index(),
        );
        set_combo_model(&imp.scaler, &Scaler::LABELS, settings.scaler.index());
        set_combo_model(&imp.filter, &Filter::LABELS, settings.filter.index());

        self.rows().apply_settings(settings, defaults);
    }
}

impl SettingsRows {
    fn apply_settings(&self, settings: &GamescopeSettings, defaults: &DisplayDefaults) {
        let output_resolution = settings.output_resolution.unwrap_or(defaults.native);
        let nested_resolution = settings.nested_resolution.unwrap_or(defaults.native);

        self.output_resolution_row
            .set_enable_expansion(settings.output_resolution.is_some());
        self.output_width.set_value(output_resolution.width as f64);
        self.output_height
            .set_value(output_resolution.height as f64);

        self.nested_resolution_row
            .set_enable_expansion(settings.nested_resolution.is_some());
        self.nested_preset
            .set_selected(defaults.preset_index_for(nested_resolution));
        self.nested_width.set_value(nested_resolution.width as f64);
        self.nested_height
            .set_value(nested_resolution.height as f64);

        self.refresh_row
            .set_enable_expansion(settings.nested_refresh.is_some());
        self.refresh
            .set_value(settings.nested_refresh.unwrap_or(60) as f64);

        self.fps_row
            .set_enable_expansion(settings.framerate_limit.is_some());
        self.fps
            .set_value(settings.framerate_limit.unwrap_or(60) as f64);

        self.window_mode.set_selected(settings.window_mode.index());
        self.scaler.set_selected(settings.scaler.index());
        self.filter.set_selected(settings.filter.index());

        self.hdr.set_active(settings.hdr);
        self.adaptive_sync.set_active(settings.adaptive_sync);
        self.mangoapp.set_active(settings.mangoapp);
        self.steam.set_active(settings.steam);
        self.extra_args.set_text(&settings.extra_args);
    }

    fn to_settings(&self) -> GamescopeSettings {
        GamescopeSettings {
            output_resolution: self
                .output_resolution_row
                .enables_expansion()
                .then(|| Resolution {
                    width: self.output_width.value() as u32,
                    height: self.output_height.value() as u32,
                }),
            nested_resolution: self
                .nested_resolution_row
                .enables_expansion()
                .then(|| Resolution {
                    width: self.nested_width.value() as u32,
                    height: self.nested_height.value() as u32,
                }),
            nested_refresh: self
                .refresh_row
                .enables_expansion()
                .then(|| self.refresh.value() as u32),
            framerate_limit: self
                .fps_row
                .enables_expansion()
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

fn build_window(
    app: &adw::Application,
    identity: &ProfileIdentity,
    settings: GamescopeSettings,
    command_preview: &str,
    outcome: Rc<RefCell<UiOutcome>>,
) {
    install_styles();
    let display_defaults = DisplayDefaults::detect().unwrap_or_default();
    let window = adw::ApplicationWindow::builder()
        .application(app)
        .title("Gamescope")
        .default_width(560)
        .default_height(720)
        .build();

    let content = GamescopeContent::new();
    content.configure(identity, &settings, command_preview, &display_defaults);
    let rows = content.rows();
    connect_resolution_preset(&rows);
    window.set_content(Some(&content));

    install_window_actions(app, &window, &rows, &display_defaults);
    install_app_actions(app, &window);

    {
        let app = app.clone();
        let window = window.clone();
        let rows = rows.clone();
        let display_defaults = display_defaults.clone();
        content
            .imp()
            .command_banner
            .connect_button_clicked(move |_| {
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

                *outcome.borrow_mut() = UiOutcome::Start {
                    settings,
                    display_defaults: display_defaults.clone(),
                };
                app.quit();
            });
    }

    window.present();
}

fn install_window_actions(
    app: &adw::Application,
    window: &adw::ApplicationWindow,
    rows: &SettingsRows,
    display_defaults: &DisplayDefaults,
) {
    let reset_action = gio::SimpleAction::new("reset", None);
    {
        let rows = rows.clone();
        let display_defaults = display_defaults.clone();
        reset_action.connect_activate(move |_, _| {
            rows.apply_settings(&GamescopeSettings::default(), &display_defaults);
        });
    }
    window.add_action(&reset_action);

    let cancel_action = gio::SimpleAction::new("cancel", None);
    {
        let app = app.clone();
        cancel_action.connect_activate(move |_, _| app.quit());
    }
    window.add_action(&cancel_action);
    app.set_accels_for_action("win.cancel", &["Escape"]);
}

fn install_app_actions(app: &adw::Application, window: &adw::ApplicationWindow) {
    if app.lookup_action("about").is_some() {
        return;
    }

    let about_action = gio::SimpleAction::new("about", None);
    let parent = window.clone();
    about_action.connect_activate(move |_, _| show_about_dialog(&parent));
    app.add_action(&about_action);
}

fn show_about_dialog(parent: &adw::ApplicationWindow) {
    let dialog = adw::AboutDialog::builder()
        .application_name("Gamescope GUI")
        .application_icon("input-gaming-symbolic")
        .developer_name("Andrii Shafar")
        .version(env!("CARGO_PKG_VERSION"))
        .website("https://github.com/andriishafar/gamescope-gui")
        .license_type(gtk::License::MitX11)
        .comments("Native GTK4 wrapper for launching games with Gamescope.")
        .build();
    dialog.present(Some(parent));
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

fn set_combo_model(row: &adw::ComboRow, labels: &[&str], selected: u32) {
    let model = gtk::StringList::new(labels);
    row.set_model(Some(&model));
    row.set_selected(selected);
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
    let (width, height) = resolution.split_once('×')?;
    Some(Resolution {
        width: width.parse().ok()?,
        height: height.parse().ok()?,
    })
}

fn show_error(parent: &adw::ApplicationWindow, heading: &str, body: &str) {
    let dialog = adw::AlertDialog::new(Some(heading), Some(body));
    dialog.add_response("ok", "OK");
    dialog.set_default_response(Some("ok"));
    dialog.set_close_response("ok");
    dialog.present(Some(parent));
}

fn install_styles() {
    let Some(display) = gdk::Display::default() else {
        return;
    };
    let provider = gtk::CssProvider::new();
    provider.load_from_string(LAUNCH_BANNER_CSS);
    gtk::style_context_add_provider_for_display(
        &display,
        &provider,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}

fn render_command_preview(command: &[OsString]) -> String {
    command
        .iter()
        .map(|arg| arg.to_string_lossy())
        .collect::<Vec<_>>()
        .join(" ")
}
