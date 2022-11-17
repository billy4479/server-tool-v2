use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};
use std::{fs::File, path::PathBuf};

#[derive(Serialize, Deserialize, Debug)]
struct ApplicationConfig {
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
struct MinecraftConfig {
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
struct JavaConfig {
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
struct GitConfig {
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
    application: ApplicationConfig,
    minecraft: MinecraftConfig,
    java: JavaConfig,
    git: GitConfig,
}

impl Config {
    pub fn config_dir() -> Result<PathBuf> {
        match dirs::config_dir() {
            Some(path) => Ok(path.join("server-tool")),
            None => bail!("no config_dir"),
        }
    }

    pub fn config_path() -> Result<PathBuf> {
        Ok(Self::config_dir()?.join("server-tool.yml"))
    }

    pub fn load() -> Result<Self> {
        Ok(serde_yaml::from_reader(File::open(Self::config_path()?)?)?)
    }

    pub fn write_default() -> Result<()> {
        match serde_yaml::to_writer(File::create(Self::config_path()?)?, &Self::default()) {
            Err(e) => Err(e.into()),
            _ => Ok(()),
        }
    }
}
