//! Wire types for sending between BE<->FE.

/// Info about a torrent file.
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Torrent {
    pub added: String,
    pub category: String,
    pub descr: Option<String>,
    pub download_count: Option<String>,
    pub id: String,
    pub info_hash: String,
    pub leechers: String,
    pub name: String,
    pub num_files: String,
    pub seeders: String,
    pub size: String,
    pub status: String,
    pub username: String,
    pub magnet: Option<String>,
}

/// Any error.
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Error {
    pub msg: String,
}

impl<T: ToString> From<T> for Error {
    fn from(value: T) -> Self {
        let msg = value.to_string();
        Self { msg }
    }
}
