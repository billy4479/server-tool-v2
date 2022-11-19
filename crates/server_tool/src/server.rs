use std::path;

use anyhow::Result;

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
    pub version: String,
    pub has_git: bool,
    pub server_type: ServerType,
}

impl Server {
    pub fn find(base_dir: &path::Path) -> Result<Vec<Server>> {
        let mut result = Vec::<Server>::new();

        for entry_result in base_dir.read_dir()? {
            let Ok(entry) = entry_result else {
                continue;
            };

            if !entry.file_type()?.is_dir() {
                continue;
            }

            if let Some(server) = Self::detect(&entry.path())? {
                result.push(server)
            }
        }

        Ok(result)
    }

    fn detect(path: &path::Path) -> Result<Option<Server>> {
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

    fn detect_version(jar_path: &path::Path) -> Result<Option<String>> {
        todo!()
    }
}
