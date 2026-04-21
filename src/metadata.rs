//! Tracks installation state and metadata for downloaded and detected tools.

use std::fs;
use zed_extension_api::{serde_json, Result};

#[derive(Debug, Clone, Default)]
pub struct InstallationState {
    pub platform: Option<Platform>,
    pub arduino_cli: Option<ToolMetadata>,
    pub clangd: Option<ToolMetadata>,
    pub arduino_language_server: Option<ToolMetadata>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Platform {
    Linux,
    MacOS,
    Windows,
}

#[derive(Debug, Clone)]
pub struct ToolMetadata {
    pub source: ToolSource,
    pub version: Option<String>,
    pub location: String,
    pub uses_isolated_data: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ToolSource {
    Downloaded,
    Path,
    ZedManaged,
    Manual,
}

impl InstallationState {
    // ============================================================================
    // Load/Save
    // ============================================================================

    /// Load installation state from disk
    pub fn load() -> Self {
        let state_file = "installation_state.json";

        if let Ok(contents) = fs::read_to_string(state_file) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&contents) {
                return Self::from_json(&json);
            }
        }

        Self::default()
    }

    /// Save installation state to disk
    pub fn save(&self) -> Result<()> {
        let state_file = "installation_state.json";
        let json = self.to_json();

        let json_string = serde_json::to_string_pretty(&json)
            .map_err(|e| format!("failed to serialize installation state: {}", e))?;

        fs::write(state_file, json_string)
            .map_err(|e| format!("failed to write installation state: {}", e))?;

        Ok(())
    }

    // ============================================================================
    // Getters/Queries
    // ============================================================================

    /// Get the detected platform
    pub fn get_platform(&self) -> Option<&Platform> {
        self.platform.as_ref()
    }

    /// Check if arduino-cli was downloaded by extension
    pub fn arduino_cli_installed_by_extension(&self) -> bool {
        self.arduino_cli
            .as_ref()
            .map(|m| m.source == ToolSource::Downloaded)
            .unwrap_or(false)
    }

    /// Check if arduino-cli uses isolated data directory
    pub fn arduino_cli_uses_isolated_data(&self) -> bool {
        self.arduino_cli
            .as_ref()
            .map(|m| m.uses_isolated_data)
            .unwrap_or(false)
    }

    /// Get arduino-cli isolated data directory path
    pub fn get_arduino_cli_data_dir(&self) -> Option<String> {
        if self.arduino_cli_uses_isolated_data() {
            Some("arduino-data".to_string())
        } else {
            None
        }
    }

    // ============================================================================
    // Mutators
    // ============================================================================

    pub fn record_platform(&mut self, platform: Platform) {
        self.platform = Some(platform);
    }

    pub fn record_arduino_cli_download(&mut self, version: &str, location: String) {
        self.arduino_cli = Some(ToolMetadata {
            source: ToolSource::Downloaded,
            version: Some(version.to_string()),
            location,
            uses_isolated_data: true,
        });
    }

    pub fn record_arduino_cli_from_path(&mut self, location: String) {
        self.arduino_cli = Some(ToolMetadata {
            source: ToolSource::Path,
            version: None,
            location,
            uses_isolated_data: false,
        });
    }

    pub fn record_clangd_download(&mut self, version: &str, location: String) {
        self.clangd = Some(ToolMetadata {
            source: ToolSource::Downloaded,
            version: Some(version.to_string()),
            location,
            uses_isolated_data: false,
        });
    }

    pub fn record_clangd_from_system(&mut self, location: String, source: ToolSource) {
        self.clangd = Some(ToolMetadata {
            source,
            version: None,
            location,
            uses_isolated_data: false,
        });
    }

    pub fn record_language_server_download(&mut self, version: &str, location: String) {
        self.arduino_language_server = Some(ToolMetadata {
            source: ToolSource::Downloaded,
            version: Some(version.to_string()),
            location,
            uses_isolated_data: false,
        });
    }

    pub fn record_language_server_manual(&mut self, location: String) {
        self.arduino_language_server = Some(ToolMetadata {
            source: ToolSource::Manual,
            version: None,
            location,
            uses_isolated_data: false,
        });
    }

    // ============================================================================
    // JSON Serialization
    // ============================================================================

    fn to_json(&self) -> serde_json::Value {
        let mut obj = serde_json::json!({});

        if let Some(ref platform) = self.platform {
            let platform_str = match platform {
                Platform::Linux => "linux",
                Platform::MacOS => "macos",
                Platform::Windows => "windows",
            };
            obj["platform"] = serde_json::Value::String(platform_str.to_string());
        }

        if let Some(ref metadata) = self.arduino_cli {
            obj["arduino_cli"] = metadata.to_json();
        }

        if let Some(ref metadata) = self.clangd {
            obj["clangd"] = metadata.to_json();
        }

        if let Some(ref metadata) = self.arduino_language_server {
            obj["arduino_language_server"] = metadata.to_json();
        }

        obj
    }

    fn from_json(json: &serde_json::Value) -> Self {
        let platform = json
            .get("platform")
            .and_then(|v| v.as_str())
            .and_then(|s| match s {
                "linux" => Some(Platform::Linux),
                "macos" => Some(Platform::MacOS),
                "windows" => Some(Platform::Windows),
                _ => None,
            });

        Self {
            platform,
            arduino_cli: json.get("arduino_cli").and_then(ToolMetadata::from_json),
            clangd: json.get("clangd").and_then(ToolMetadata::from_json),
            arduino_language_server: json
                .get("arduino_language_server")
                .and_then(ToolMetadata::from_json),
        }
    }
}

impl ToolMetadata {
    fn to_json(&self) -> serde_json::Value {
        let source_str = match self.source {
            ToolSource::Downloaded => "downloaded",
            ToolSource::Path => "path",
            ToolSource::ZedManaged => "zed_managed",
            ToolSource::Manual => "manual",
        };

        let mut obj = serde_json::json!({
            "source": source_str,
            "location": self.location,
            "uses_isolated_data": self.uses_isolated_data,
        });

        if let Some(ref version) = self.version {
            obj["version"] = serde_json::Value::String(version.clone());
        }

        obj
    }

    fn from_json(json: &serde_json::Value) -> Option<Self> {
        let source_str = json.get("source")?.as_str()?;
        let source = match source_str {
            "downloaded" => ToolSource::Downloaded,
            "path" => ToolSource::Path,
            "zed_managed" => ToolSource::ZedManaged,
            "manual" => ToolSource::Manual,
            _ => return None,
        };

        let location = json.get("location")?.as_str()?.to_string();
        let uses_isolated_data = json
            .get("uses_isolated_data")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let version = json
            .get("version")
            .and_then(|v| v.as_str())
            .map(String::from);

        Some(Self {
            source,
            version,
            location,
            uses_isolated_data,
        })
    }
}
