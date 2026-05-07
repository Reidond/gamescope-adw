use std::collections::BTreeMap;
use std::ffi::{OsStr, OsString};
use std::fs;
use std::io;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

#[cfg(unix)]
use std::os::unix::ffi::OsStrExt;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::settings::GamescopeSettings;

const APP_PREFIX: &str = "gamescope-gui";
const PROFILES_FILE: &str = "profiles.toml";
const CURRENT_SCHEMA_VERSION: u32 = 2;
const STEAM_ID_ENV: [&str; 3] = ["STEAM_COMPAT_APP_ID", "SteamAppId", "SteamGameId"];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProfileIdentity {
    pub key: String,
    pub label: String,
    pub command_hash: String,
    pub is_steam_profile: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GameProfile {
    pub label: String,
    pub command_hash: String,
    pub updated_at_unix_secs: u64,
    pub settings: GamescopeSettings,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProfileStore {
    #[serde(default)]
    pub schema_version: u32,
    #[serde(default)]
    pub profiles: BTreeMap<String, GameProfile>,
}

impl Default for ProfileStore {
    fn default() -> Self {
        Self {
            schema_version: CURRENT_SCHEMA_VERSION,
            profiles: BTreeMap::new(),
        }
    }
}

#[derive(Debug, Error)]
pub enum ProfileError {
    #[error("could not resolve XDG profile path: {0}")]
    Xdg(#[from] io::Error),
    #[error("could not read profiles from {path}: {source}")]
    Read { path: PathBuf, source: io::Error },
    #[error("could not parse profiles from {path}: {source}")]
    Parse {
        path: PathBuf,
        source: toml::de::Error,
    },
    #[error("could not serialize profiles: {0}")]
    Serialize(#[from] toml::ser::Error),
    #[error("could not write profiles to {path}: {source}")]
    Write { path: PathBuf, source: io::Error },
}

impl ProfileIdentity {
    pub fn from_current_env(game_command: &[OsString]) -> Self {
        Self::from_env_lookup(game_command, |key| std::env::var_os(key))
    }

    pub fn from_env_lookup<F>(game_command: &[OsString], mut env_lookup: F) -> Self
    where
        F: FnMut(&str) -> Option<OsString>,
    {
        let command_hash = command_hash(game_command);
        for key in STEAM_ID_ENV {
            if let Some(value) = env_lookup(key) {
                let value = value.to_string_lossy();
                let trimmed = value.trim();
                if !trimmed.is_empty() {
                    return Self {
                        key: format!("steam-{trimmed}"),
                        label: format!("Steam App {trimmed}"),
                        command_hash,
                        is_steam_profile: true,
                    };
                }
            }
        }

        let label = game_command
            .first()
            .and_then(|arg| {
                std::path::Path::new(arg)
                    .file_name()
                    .map(|name| name.to_string_lossy().into_owned())
            })
            .filter(|label| !label.is_empty())
            .unwrap_or_else(|| "Non-Steam game".to_string());

        Self {
            key: format!("command-{command_hash}"),
            label,
            command_hash,
            is_steam_profile: false,
        }
    }
}

impl ProfileStore {
    pub fn load() -> Result<Self, ProfileError> {
        let path = profile_path(false)?;
        Self::load_from_path(path)
    }

    pub fn save(&self) -> Result<PathBuf, ProfileError> {
        let path = profile_path(true)?;
        self.save_to_path(path.clone())?;
        Ok(path)
    }

    pub fn load_from_path(path: PathBuf) -> Result<Self, ProfileError> {
        match fs::read_to_string(&path) {
            Ok(contents) => toml::from_str(&contents)
                .map(migrate_store)
                .map_err(|source| ProfileError::Parse { path, source }),
            Err(source) if source.kind() == io::ErrorKind::NotFound => Ok(Self::default()),
            Err(source) => Err(ProfileError::Read { path, source }),
        }
    }

    pub fn save_to_path(&self, path: PathBuf) -> Result<(), ProfileError> {
        let contents = toml::to_string_pretty(self)?;
        fs::write(&path, contents).map_err(|source| ProfileError::Write { path, source })
    }

    pub fn settings_for(&self, identity: &ProfileIdentity) -> GamescopeSettings {
        if let Some(profile) = self.profiles.get(&identity.key) {
            return profile.settings.clone();
        }

        GamescopeSettings::default()
    }

    pub fn upsert(&mut self, identity: &ProfileIdentity, settings: GamescopeSettings) {
        self.profiles.insert(
            identity.key.clone(),
            GameProfile {
                label: identity.label.clone(),
                command_hash: identity.command_hash.clone(),
                updated_at_unix_secs: unix_time_now(),
                settings,
            },
        );
    }
}

fn migrate_store(mut store: ProfileStore) -> ProfileStore {
    if store.schema_version < 2 {
        for profile in store.profiles.values_mut() {
            profile.settings.steam = false;
        }
    }
    store.schema_version = CURRENT_SCHEMA_VERSION;
    store
}

pub fn profile_path(create_parent: bool) -> Result<PathBuf, io::Error> {
    let dirs = xdg::BaseDirectories::with_prefix(APP_PREFIX);
    if create_parent {
        dirs.place_config_file(PROFILES_FILE)
    } else {
        dirs.get_config_home()
            .map(|path| path.join(PROFILES_FILE))
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "$HOME must be set"))
    }
}

pub fn command_hash(command: &[OsString]) -> String {
    let mut hasher = blake3::Hasher::new();
    for arg in command {
        update_hash_with_os_str(&mut hasher, arg.as_os_str());
        hasher.update(&[0]);
    }
    hasher.finalize().to_hex()[..16].to_string()
}

#[cfg(unix)]
fn update_hash_with_os_str(hasher: &mut blake3::Hasher, value: &OsStr) {
    hasher.update(value.as_bytes());
}

#[cfg(not(unix))]
fn update_hash_with_os_str(hasher: &mut blake3::Hasher, value: &OsStr) {
    hasher.update(value.to_string_lossy().as_bytes());
}

fn unix_time_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn os_args(args: &[&str]) -> Vec<OsString> {
        args.iter().map(OsString::from).collect()
    }

    #[test]
    fn prefers_steam_compat_app_id() {
        let command = os_args(&["game"]);
        let identity = ProfileIdentity::from_env_lookup(&command, |key| match key {
            "STEAM_COMPAT_APP_ID" => Some("123".into()),
            "SteamAppId" => Some("456".into()),
            _ => None,
        });

        assert_eq!(identity.key, "steam-123");
        assert_eq!(identity.label, "Steam App 123");
        assert!(identity.is_steam_profile);
    }

    #[test]
    fn falls_back_to_command_hash() {
        let command = os_args(&["/opt/Games/My Game/run.sh", "--flag"]);
        let identity = ProfileIdentity::from_env_lookup(&command, |_| None);

        assert!(identity.key.starts_with("command-"));
        assert_eq!(identity.label, "run.sh");
        assert_eq!(identity.command_hash, command_hash(&command));
        assert!(!identity.is_steam_profile);
    }

    #[test]
    fn default_settings_do_not_enable_gamescope_steam_mode() {
        let store = ProfileStore::default();
        let steam_identity = ProfileIdentity::from_env_lookup(&os_args(&["game"]), |key| {
            (key == "SteamAppId").then(|| "42".into())
        });
        let command_identity = ProfileIdentity::from_env_lookup(&os_args(&["game"]), |_| None);

        assert!(!store.settings_for(&steam_identity).steam);
        assert!(!store.settings_for(&command_identity).steam);
    }

    #[test]
    fn round_trips_profiles() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("profiles.toml");
        let identity = ProfileIdentity {
            key: "steam-42".to_string(),
            label: "Steam App 42".to_string(),
            command_hash: "abc".to_string(),
            is_steam_profile: true,
        };
        let mut store = ProfileStore::default();
        store.upsert(&identity, GamescopeSettings::default());

        store.save_to_path(path.clone()).unwrap();
        let loaded = ProfileStore::load_from_path(path).unwrap();

        assert_eq!(loaded.profiles.len(), 1);
        assert_eq!(
            loaded.profiles["steam-42"].settings,
            GamescopeSettings::default()
        );
    }

    #[test]
    fn migrates_old_implicit_steam_profiles() {
        let old_store = r#"
[profiles.steam-999999]
label = "Steam App 999999"
command_hash = "abc"
updated_at_unix_secs = 1

[profiles.steam-999999.settings]
window_mode = "Windowed"
scaler = "Default"
filter = "Default"
hdr = false
adaptive_sync = false
mangoapp = false
steam = true
extra_args = ""
"#;
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("profiles.toml");
        fs::write(&path, old_store).unwrap();

        let loaded = ProfileStore::load_from_path(path).unwrap();

        assert_eq!(loaded.schema_version, CURRENT_SCHEMA_VERSION);
        assert!(!loaded.profiles["steam-999999"].settings.steam);
    }
}
