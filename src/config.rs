use rs_qq::client::protocol::device::Device;
use rs_qq::Config as QQConfig;
use serde::{Deserialize, Serialize};
use std::io::Read;
use tracing::{info, warn};
use walle_core::ImplConfig;

type IOResult<T> = Result<T, std::io::Error>;

const CONFIG_PATH: &str = "walle-q.yaml";
const DEVICE_PATH: &str = "device.json";

#[derive(Debug, Default, Serialize, Deserialize)]
pub(crate) struct Config {
    pub onebot: ImplConfig,
    pub qq: QQConfig,
}

trait NewConfig: Sized {
    fn new_config() -> Self;
    fn ser(&self) -> IOResult<String>;
    fn de(s: &str) -> IOResult<Self>;
}

trait LoadConfig: for<'de> Deserialize<'de> + Serialize + NewConfig {
    fn save_to_file(&self, path: &str) -> IOResult<()> {
        let data = self.ser()?;
        std::fs::write(path, data)
    }

    fn load_from_file(path: &str) -> IOResult<Self> {
        let mut file = std::fs::File::open(path)?;
        let mut data = String::new();
        file.read_to_string(&mut data)?;
        Self::de(&data)
    }

    fn load_or_new<F>(path: &str, f: F) -> IOResult<Self>
    where
        F: FnOnce(&mut Self) -> IOResult<()>,
    {
        info!("loading {}", path);
        let mut config = match Self::load_from_file(path) {
            Ok(config) => {
                info!("success load from {}", path);
                config
            }
            Err(e) => match e.kind() {
                std::io::ErrorKind::Other => {
                    warn!("Serialize error: {}", e);
                    return Err(e);
                }
                _ => {
                    warn!("open {} failed: {}", path, e);
                    info!("creating new {}", path);
                    let config = Self::new_config();
                    config.save_to_file(path)?;
                    config
                }
            },
        };
        f(&mut config)?;
        Ok(config)
    }
}

impl NewConfig for Config {
    fn new_config() -> Self {
        Self::default()
    }

    fn ser(&self) -> IOResult<String> {
        serde_yaml::to_string(self).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
    }

    fn de(s: &str) -> IOResult<Self> {
        serde_yaml::from_str(s).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
    }
}

impl LoadConfig for Config {}

impl NewConfig for Device {
    fn new_config() -> Self {
        Self::random()
    }

    fn ser(&self) -> IOResult<String> {
        serde_json::to_string(self).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
    }

    fn de(s: &str) -> IOResult<Self> {
        serde_json::from_str(s).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
    }
}

impl LoadConfig for Device {}

impl Config {
    pub(crate) fn load() -> Result<Self, std::io::Error> {
        Self::load_or_new(CONFIG_PATH, |config| {
            config.qq.device = Device::load_or_new(DEVICE_PATH, |_| Ok(()))?;
            Ok(())
        })
    }
}
