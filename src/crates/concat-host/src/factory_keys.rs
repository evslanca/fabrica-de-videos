// SPDX-License-Identifier: AGPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Concat contributors

//! Secure API key store for optional cloud providers (OpenAI, ElevenLabs, Pexels).
//!
//! Keys are persisted to the app's config directory (`AppDirs::config`)
//! in a simple JSON file. They are loaded lazily at first use and stored
//! without further encryption beyond the OS user-profile boundary.
//! When a key is absent the factory falls back to free/local providers
//! and the app functions 100% without any keys configured.

use std::fs;
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};

/// File name within the config directory.
const KEYS_FILE: &str = "factory_keys.json";

/// Optional cloud provider keys.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FactoryKeys {
    /// OpenAI secret key (tts-1 / tts-1-hd).
    pub openai: Option<String>,
    /// ElevenLabs API key.
    pub elevenlabs: Option<String>,
    /// Pexels API key (extends free quota per IP).
    pub pexels: Option<String>,
    /// Pixabay API key (optional — public endpoint available without key).
    pub pixabay: Option<String>,
}

impl FactoryKeys {
    /// Returns `true` when at least one key is configured.
    pub fn any_configured(&self) -> bool {
        self.openai.is_some()
            || self.elevenlabs.is_some()
            || self.pexels.is_some()
            || self.pixabay.is_some()
    }

    /// Loads keys from the given config directory.
    /// Returns a default (all `None`) instance if the file does not exist.
    pub fn load(config_dir: &Path) -> Self {
        let path = Self::path(config_dir);
        match fs::read_to_string(&path) {
            Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
            Err(_) => Self::default(),
        }
    }

    /// Persists keys to the config directory.
    pub fn save(&self, config_dir: &Path) -> std::io::Result<()> {
        let path = Self::path(config_dir);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        fs::write(&path, json)
    }

    /// Sets a key by provider name ("openai", "elevenlabs", "pexels", "pixabay").
    /// Clears the key when value is empty.
    pub fn set(&mut self, provider: &str, value: String) {
        let val = if value.trim().is_empty() { None } else { Some(value) };
        match provider {
            "openai"     => self.openai = val,
            "elevenlabs" => self.elevenlabs = val,
            "pexels"     => self.pexels = val,
            "pixabay"    => self.pixabay = val,
            _ => log::warn!("FactoryKeys::set: unknown provider '{}'", provider),
        }
    }

    fn path(config_dir: &Path) -> PathBuf {
        config_dir.join(KEYS_FILE)
    }
}
