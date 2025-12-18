use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tauri::Manager;

/// Threshold configuration for a single metric
/// From specs/module-plan-method-vi.md (line 3119-3131)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricThreshold {
    pub pass: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub warning: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub halt: Option<f64>,
}

/// Critical 6 metrics thresholds
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Critical6Thresholds {
    #[serde(rename = "CI")]
    pub ci: MetricThreshold,
    #[serde(rename = "EV")]
    pub ev: MetricThreshold,
    #[serde(rename = "IAS")]
    pub ias: MetricThreshold,
    #[serde(rename = "EFI")]
    pub efi: MetricThreshold,
    #[serde(rename = "SEC")]
    pub sec: MetricThreshold,
    #[serde(rename = "PCI")]
    pub pci: MetricThreshold,
}

/// Advisory 5 metrics thresholds (Phase 2)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Advisory5Thresholds {
    #[serde(rename = "GLR")]
    pub glr: MetricThreshold,
    #[serde(rename = "RCC")]
    pub rcc: MetricThreshold,
    #[serde(rename = "CAI")]
    pub cai: MetricThreshold,
    #[serde(rename = "RUV")]
    pub ruv: MetricThreshold,
    #[serde(rename = "LLE")]
    pub lle: MetricThreshold,
}

/// Mode-specific thresholds
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SurgicalModeThresholds {
    pub max_patches: i32,
    pub cumulative_ev_limit: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModeSpecificThresholds {
    pub surgical: SurgicalModeThresholds,
}

/// Complete threshold configuration
/// From specs/module-plan-method-vi.md (line 3113-3152)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThresholdConfig {
    pub version: String,
    pub source: String,
    pub critical_6: Critical6Thresholds,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub advisory_5: Option<Advisory5Thresholds>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mode_specific: Option<ModeSpecificThresholds>,
}

impl Default for ThresholdConfig {
    /// Default thresholds from Method-VI Core v1.0.1
    /// From specs/module-plan-method-vi.md (line 3119-3152)
    fn default() -> Self {
        ThresholdConfig {
            version: "1.0.0".to_string(),
            source: "Method-VI Core v1.0.1".to_string(),
            critical_6: Critical6Thresholds {
                ci: MetricThreshold {
                    pass: 0.80,
                    warning: Some(0.70),
                    halt: Some(0.50),
                },
                ev: MetricThreshold {
                    pass: 10.0,
                    warning: Some(20.0),
                    halt: Some(30.0),
                },
                ias: MetricThreshold {
                    pass: 0.80,
                    warning: Some(0.70),
                    halt: Some(0.50),
                },
                efi: MetricThreshold {
                    pass: 95.0,
                    warning: Some(90.0),
                    halt: Some(80.0),
                },
                sec: MetricThreshold {
                    pass: 100.0,
                    warning: None,
                    halt: None,
                },
                pci: MetricThreshold {
                    pass: 0.90,
                    warning: Some(0.85),
                    halt: Some(0.70),
                },
            },
            advisory_5: Some(Advisory5Thresholds {
                glr: MetricThreshold {
                    pass: 15.0,
                    warning: None,
                    halt: None,
                },
                rcc: MetricThreshold {
                    pass: 0.85,
                    warning: None,
                    halt: None,
                },
                cai: MetricThreshold {
                    pass: 0.80,
                    warning: None,
                    halt: None,
                },
                ruv: MetricThreshold {
                    pass: 0.75,
                    warning: None,
                    halt: None,
                },
                lle: MetricThreshold {
                    pass: 0.70,
                    warning: None,
                    halt: None,
                },
            }),
            mode_specific: Some(ModeSpecificThresholds {
                surgical: SurgicalModeThresholds {
                    max_patches: 5,
                    cumulative_ev_limit: 15.0,
                },
            }),
        }
    }
}

impl ThresholdConfig {
    /// Get path to thresholds.json config file
    /// From specs/module-plan-method-vi.md (line 3117)
    pub fn get_config_path(app_handle: &tauri::AppHandle) -> Result<PathBuf> {
        let app_data_dir = app_handle
            .path()
            .app_data_dir()
            .context("Failed to get app data directory")?;

        let config_dir = app_data_dir.join("config");
        std::fs::create_dir_all(&config_dir)
            .context("Failed to create config directory")?;

        Ok(config_dir.join("thresholds.json"))
    }

    /// Load thresholds from config file with fallback to defaults
    /// From specs/Method-VI_Test_Case_Specifications.md TC-TH-001 (line 670-679)
    pub fn load(app_handle: &tauri::AppHandle) -> Self {
        let config_path = match Self::get_config_path(app_handle) {
            Ok(path) => path,
            Err(e) => {
                log::warn!("Failed to get config path: {}", e);
                log::info!("Using default thresholds");
                return Self::default();
            }
        };

        // Try to read existing config file
        match std::fs::read_to_string(&config_path) {
            Ok(contents) => {
                // Try to parse JSON
                match serde_json::from_str::<ThresholdConfig>(&contents) {
                    Ok(config) => {
                        log::info!("Loaded thresholds from config file: {:?}", config_path);
                        // Validate config version
                        if config.version != "1.0.0" {
                            log::warn!(
                                "Config version mismatch: expected 1.0.0, got {}. Using anyway.",
                                config.version
                            );
                        }
                        config
                    }
                    Err(e) => {
                        // Corrupted config - use defaults
                        log::error!("Failed to parse thresholds config: {}", e);
                        log::info!("Using default thresholds (config corrupted)");
                        Self::default()
                    }
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                // Config file doesn't exist - create it with defaults
                log::info!("Threshold config not found, creating default config");
                let default_config = Self::default();

                // Try to write default config
                if let Err(write_err) = default_config.save(app_handle) {
                    log::warn!("Failed to save default threshold config: {}", write_err);
                }

                default_config
            }
            Err(e) => {
                // Other IO error - use defaults
                log::error!("Failed to read threshold config: {}", e);
                log::info!("Using default thresholds");
                Self::default()
            }
        }
    }

    /// Save thresholds to config file
    pub fn save(&self, app_handle: &tauri::AppHandle) -> Result<()> {
        let config_path = Self::get_config_path(app_handle)?;

        let json = serde_json::to_string_pretty(self)
            .context("Failed to serialize threshold config")?;

        std::fs::write(&config_path, json)
            .context("Failed to write threshold config file")?;

        log::info!("Saved threshold config to: {:?}", config_path);
        Ok(())
    }

    /// Get threshold for a specific metric by name
    pub fn get_metric_threshold(&self, metric_name: &str) -> Option<&MetricThreshold> {
        match metric_name {
            "CI" => Some(&self.critical_6.ci),
            "EV" => Some(&self.critical_6.ev),
            "IAS" => Some(&self.critical_6.ias),
            "EFI" => Some(&self.critical_6.efi),
            "SEC" => Some(&self.critical_6.sec),
            "PCI" => Some(&self.critical_6.pci),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_thresholds() {
        let config = ThresholdConfig::default();

        // Test CI thresholds
        assert_eq!(config.critical_6.ci.pass, 0.80);
        assert_eq!(config.critical_6.ci.warning, Some(0.70));
        assert_eq!(config.critical_6.ci.halt, Some(0.50));

        // Test EV thresholds
        assert_eq!(config.critical_6.ev.pass, 10.0);
        assert_eq!(config.critical_6.ev.warning, Some(20.0));
        assert_eq!(config.critical_6.ev.halt, Some(30.0));

        // Test SEC (no warning/halt)
        assert_eq!(config.critical_6.sec.pass, 100.0);
        assert_eq!(config.critical_6.sec.warning, None);
        assert_eq!(config.critical_6.sec.halt, None);
    }

    #[test]
    fn test_serialize_deserialize() {
        let config = ThresholdConfig::default();

        // Serialize to JSON
        let json = serde_json::to_string_pretty(&config).unwrap();

        // Deserialize back
        let deserialized: ThresholdConfig = serde_json::from_str(&json).unwrap();

        // Verify values match
        assert_eq!(deserialized.critical_6.ci.pass, config.critical_6.ci.pass);
        assert_eq!(deserialized.critical_6.ev.pass, config.critical_6.ev.pass);
    }

    #[test]
    fn test_get_metric_threshold() {
        let config = ThresholdConfig::default();

        // Test valid metric
        let ci_threshold = config.get_metric_threshold("CI");
        assert!(ci_threshold.is_some());
        assert_eq!(ci_threshold.unwrap().pass, 0.80);

        // Test invalid metric
        let invalid = config.get_metric_threshold("INVALID");
        assert!(invalid.is_none());
    }
}
