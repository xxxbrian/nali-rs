use serde::Deserialize;
use std::{error::Error, fs};

#[derive(Debug)]
pub struct NaliConfig {
    app_support_path: String,
    toml_config: TomlConfig,
}

impl Default for NaliConfig {
    fn default() -> Self {
        Self::new()
    }
}

impl NaliConfig {
    pub fn new() -> Self {
        let app_support_path = dirs::config_dir().unwrap().join("nali-rs");

        // Use the default configuration if the config file is missing
        let toml_config =
            match TomlConfig::read_config(app_support_path.join("config.toml").to_str().unwrap()) {
                Ok(config) => config,
                Err(e) => {
                    println!("Failed to read config file: {}", e);
                    println!("Using default configuration");
                    TomlConfig::default()
                }
            };

        Self {
            app_support_path: app_support_path.to_str().unwrap().to_string(),
            toml_config,
        }
    }

    pub fn parser(&self) -> ParserConfig {
        self.toml_config.parser.clone()
    }

    pub fn geodb(&self) -> GeoDBConfig {
        self.toml_config.geodb.clone()
    }

    pub fn app_support_path(&self) -> &str {
        &self.app_support_path
    }
}

// TOML Config
#[derive(Deserialize, Debug, Default)]
#[serde(default, deny_unknown_fields)]
struct TomlConfig {
    parser: ParserConfig,
    geodb: GeoDBConfig,
}

impl TomlConfig {
    fn read_config(path: &str) -> Result<TomlConfig, Box<dyn Error>> {
        let contents = fs::read_to_string(path)?;
        let config: TomlConfig = toml::from_str(&contents)?;
        Ok(config)
    }
}

#[derive(Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum ParserConfig {
    #[default]
    FastParser,
    RegexParser,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum GeoDBConfig {
    GeoLite2(GeoLite2Config),
    FakeGeo(FakeGeoConfig),
}

impl Default for GeoDBConfig {
    fn default() -> Self {
        GeoDBConfig::GeoLite2(GeoLite2Config::default())
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct GeoLite2Config {
    pub path: String,
}

impl Default for GeoLite2Config {
    fn default() -> Self {
        GeoLite2Config {
            path: "GeoLite2-City.mmdb".to_string(),
        }
    }
}

impl GeoLite2Config {
    pub fn full_path(&self, app_support_path: &str) -> String {
        format!("{}/{}", app_support_path, self.path)
    }
}
#[derive(Deserialize, Debug, Clone, Default)]
pub struct FakeGeoConfig {}
