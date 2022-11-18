use std::path;

use anyhow::Result;

pub struct Server {
    name: String,
    path: path::PathBuf,
    version: String,
}
