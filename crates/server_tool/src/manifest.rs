use std::{
    fs::{self, File},
    path::PathBuf,
};

use anyhow::{anyhow, bail, Error, Result};
use futures::future;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct VersionManifest {
    id: String,
    jar_url: String,
    java_version: u64,
    sha1: String,
}

fn get_manifest_path() -> Result<PathBuf> {
    match dirs::data_local_dir() {
        Some(dir) => {
            let data_dir_path = dir.join("server-tool");
            fs::create_dir_all(&data_dir_path)?;
            Ok(data_dir_path.join("manifest.json"))
        }
        None => bail!("no cache directory"),
    }
}

pub async fn get_version_infos() -> Result<Vec<VersionManifest>> {
    fs::create_dir_all(dirs::data_local_dir().expect("missing data dir"))?;

    let path = get_manifest_path()?;
    match fs::metadata(&path) {
        Ok(meta) => {
            // TODO: check expired manifest
            log::info!("Version manifest was found, loading it.");
            Ok(serde_json::from_reader(File::open(path)?)?)
        }
        Err(e) => {
            log::warn!(
                "An error occurred while gathering metadata about the version manifest file: {}",
                e
            );

            update_manifest().await
        }
    }
}

pub async fn update_manifest() -> Result<Vec<VersionManifest>> {
    let manifest = download_manifest().await?;
    serde_json::to_writer_pretty(File::create(get_manifest_path()?)?, &manifest)?;
    Ok(manifest)
}

async fn download_manifest() -> Result<Vec<VersionManifest>> {
    const MANIFEST_URL: &str = "https://launchermeta.mojang.com/mc/game/version_manifest.json";

    log::info!("Downloading version manifest...");

    let value: serde_json::Value =
        serde_json::from_str(reqwest::get(MANIFEST_URL).await?.text().await?.as_str())?;

    let version_manifests_urls_lambda = move || {
        let mut result = vec![];

        for version in value["versions"].as_array()? {
            result.push(version.as_object()?["url"].as_str()?.to_string())
        }

        Some(result)
    };

    let version_manifests_urls = match version_manifests_urls_lambda() {
        Some(result) => Ok::<Vec<String>, Error>(result),
        None => bail!("error parsing json"),
    }?;

    let mut tasks = vec![];
    for manifest_url in version_manifests_urls {
        tasks.push(tokio::spawn(async move {
            let value: serde_json::Value =
                serde_json::from_str(reqwest::get(manifest_url).await?.text().await?.as_str())?;

            let java_version = (|| {
                value
                    .get("javaVersion")?
                    .as_object()?
                    .get("majorVersion")?
                    .as_u64()
            })()
            .unwrap_or(8); // Minecraft versions before 1.7 don't have a Java version specified
                           // but it should be Java 8.

            let id = value["id"]
                .as_str()
                .ok_or_else(|| anyhow!("error parsing version id"))?
                .to_string();

            let (jar_url, sha1) = match (|| -> Option<(String, String)> {
                let server = value
                    .get("downloads")?
                    .as_object()?
                    .get("server")?
                    .as_object()?;
                let jar_url = server.get("url")?.as_str()?.to_string();
                let sha1 = server.get("sha1")?.as_str()?.to_string();

                Some((jar_url, sha1))
            })() {
                Some(tup) => tup,
                None => return Ok::<Option<VersionManifest>, Error>(None),
            };

            let manifest = VersionManifest {
                id,
                jar_url,
                java_version,
                sha1,
            };

            Ok(Some(manifest))
        }));
    }

    let mut result = vec![];
    for v in future::join_all(tasks).await {
        if let Some(v) = v?? {
            result.push(v)
        }
    }

    log::info!("Download completed.");
    Ok(result)
}
