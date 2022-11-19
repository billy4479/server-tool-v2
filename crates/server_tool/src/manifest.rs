use std::{
    fs::{self, File},
    path::PathBuf,
};

use anyhow::{bail, Error, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct VersionManifest {
    id: String,
    jar_url: String,
    java_version: u64,
    sha1: String,
}

impl VersionManifest {
    fn get_manifest_path() -> Result<PathBuf> {
        match dirs::data_local_dir() {
            Some(dir) => Ok(dir.join("server-tool").join("manifest.json")),
            None => bail!("no cache directory"),
        }
    }

    pub fn get_version_infos() -> Result<Vec<VersionManifest>> {
        fs::create_dir_all(dirs::data_local_dir().expect("missing data dir"))?;

        let path = Self::get_manifest_path()?;
        match fs::metadata(&path) {
            Ok(meta) => {
                // TODO: check expired manifest
                Ok(serde_json::from_reader(File::open(path)?)?)
            }
            Err(e) => {
                let manifest = Self::download_manifest()?;
                serde_json::to_writer_pretty(File::create(&path)?, &manifest)?;
                Ok(manifest)
            }
        }
    }

    fn download_manifest() -> Result<Vec<VersionManifest>> {
        const MANIFEST_URL: &str = "https://launchermeta.mojang.com/mc/game/version_manifest.json";
        let value: serde_json::Value =
            serde_json::from_reader(reqwest::blocking::get(MANIFEST_URL)?)?;

        let version_manifests_urls_lambda = || -> Option<Vec<&str>> {
            let mut result = Vec::<&str>::new();

            for version in value["versions"].as_array()? {
                result.push(version.as_object()?["url"].as_str()?)
            }

            Some(result)
        };

        let version_manifests_urls = match version_manifests_urls_lambda() {
            Some(result) => Ok::<Vec<&str>, Error>(result),
            None => bail!("error parsing json"),
        }?;

        let mut result = Vec::<VersionManifest>::new();

        // TODO: parallelize here?
        for manifest_url in version_manifests_urls {
            let value: serde_json::Value =
                serde_json::from_reader(reqwest::blocking::get(manifest_url)?)?;

            let lambda = || -> Option<VersionManifest> {
                let java_version = value["javaVersion"].as_object()?["majorVersion"].as_u64()?;

                let id = value["id"].as_str()?.to_string();

                let server = value["downloads"].as_object()?["server"].as_object()?;
                let sha1 = server["sha1"].as_str()?.to_string();
                let jar_url = server["url"].as_str()?.to_string();

                Some(VersionManifest {
                    id,
                    jar_url,
                    java_version,
                    sha1,
                })
            };

            let manifest = match lambda() {
                Some(manifest) => Ok::<VersionManifest, Error>(manifest),
                None => bail!("error while parsing json"),
            }?;

            println!("{:?}", manifest);
            result.push(manifest);
        }

        Ok(result)
    }
}
