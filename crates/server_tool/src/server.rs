use std::{fs, path};

use anyhow::Result;
use sha1::{Digest, Sha1};

use crate::manifest::{get_version_infos, VersionManifest};

#[derive(Default, Debug)]
pub enum ServerType {
    #[default]
    Vanilla,
    Fabric,
}

#[derive(Default, Debug)]
pub struct Server {
    pub name: String,
    pub path: path::PathBuf,
    pub version: VersionManifest,
    pub has_git: bool,
    pub server_type: ServerType,
}

impl Server {
    pub async fn find(base_dir: &path::Path) -> Result<Vec<Server>> {
        let mut result = Vec::<Server>::new();
        let versions = get_version_infos().await?;

        for entry_result in base_dir.read_dir()? {
            let Ok(entry) = entry_result else {
                continue;
            };

            if !entry.file_type()?.is_dir() {
                continue;
            }

            if let Some(server) = Self::detect(&entry.path(), &versions)? {
                result.push(server)
            }
        }

        Ok(result)
    }

    fn detect(path: &path::Path, versions: &Vec<VersionManifest>) -> Result<Option<Server>> {
        const MINECRAFT_JAR: &str = "server.jar";
        const FABRIC_JAR: &str = "fabric-server-launch.jar";
        const GIT_FOLDER: &str = ".git";

        let mut result: Server = Default::default();
        let mut found = false;

        for possible_entry in path.read_dir()? {
            let Ok(entry) = possible_entry else {
                continue;
            };

            if entry.file_type()?.is_dir() {
                result.has_git = entry.file_name() == GIT_FOLDER;
            } else if entry.file_name() == MINECRAFT_JAR {
                result.name = path
                    .file_name()
                    .expect("no name")
                    .to_str()
                    .expect("string conversion")
                    .to_string();

                let file = fs::read(entry.path())?;
                let mut hasher = Sha1::new();
                hasher.update(file);
                let hash = hex::encode(hasher.finalize());

                for version in versions {
                    if version.sha1 == hash {
                        result.version = version.clone();
                        found = true;
                        break;
                    }
                }

                if !found {
                    continue;
                }
            } else if entry.file_name() == FABRIC_JAR {
                result.server_type = ServerType::Fabric;
            }
        }

        if found {
            Ok(Some(result))
        } else {
            Ok(None)
        }
    }
}
