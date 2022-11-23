use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, File},
    path::PathBuf,
};

#[derive(Serialize, Deserialize, Debug)]
pub struct ApplicationConfig {
    quiet: bool,
    working_dir: PathBuf,
    cache_dir: PathBuf,
}

impl Default for ApplicationConfig {
    fn default() -> Self {
        Self {
            quiet: false,
            working_dir: ".".into(),
            cache_dir: dirs::data_local_dir().expect("no cache dir"),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MinecraftConfig {
    quiet: bool,
    gui: bool,
    accept_eula: bool,
}

impl Default for MinecraftConfig {
    fn default() -> Self {
        Self {
            quiet: false,
            gui: true,
            accept_eula: true,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct JavaConfig {
    executable_override: Option<PathBuf>,
    memory_megabytes: u64,
    extra_flags: Vec<String>,
    override_default_flags: bool,
}

impl Default for JavaConfig {
    fn default() -> Self {
        Self {
            executable_override: None,
            memory_megabytes: 6 * 1024,
            extra_flags: Vec::new(),
            override_default_flags: false,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GitConfig {
    enabled: bool,
    use_lockfile: bool,
    pre_commands: Vec<String>,
    post_commands: Vec<String>,
}

impl Default for GitConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            use_lockfile: true,
            pre_commands: Vec::new(),
            post_commands: Vec::new(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Config {
    pub application: ApplicationConfig,
    pub minecraft: MinecraftConfig,
    pub java: JavaConfig,
    pub git: GitConfig,
}

impl Config {
    pub fn config_dir() -> Result<PathBuf> {
        match dirs::config_dir() {
            Some(path) => Ok(path.join("server-tool")),
            None => bail!("no config_dir"),
        }
    }

    pub fn config_path() -> Result<PathBuf> {
        Ok(Self::config_dir()?.join("server-tool-v2.yml"))
    }

    pub fn load() -> Result<Self> {
        fs::create_dir_all(Self::config_dir()?)?;

        log::info!("Loading config...");

        match File::open(Self::config_path()?) {
            Ok(file) => Ok(serde_yaml::from_reader(file)?),
            Err(e) => {
                log::warn!(
                    "An error occurred while loading the config, falling back on the default one. {}", e
                );
                Ok(Config::default())
            }
        }
    }

    pub fn write(&self) -> Result<()> {
        fs::create_dir_all(Self::config_dir()?)?;
        match serde_yaml::to_writer(File::create(Self::config_path()?)?, &self) {
            Err(e) => Err(e.into()),
            _ => Ok(()),
        }
    }

    pub fn to_yaml(&self) -> Result<String> {
        Ok(serde_yaml::to_string(&self)?)
    }
}
