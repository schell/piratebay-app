use tokio::sync::Mutex;

use pb_wire_types::{Error, Torrent};
use piratebay::pirateclient::PirateClient;
use tauri::{Manager, State};

struct App {
    client: PirateClient,
}

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

fn pb_torrent_to_wire(pb_t: piratebay::types::Torrent) -> Torrent {
    let piratebay::types::Torrent {
        added,
        category,
        descr,
        download_count,
        id,
        info_hash,
        leechers,
        name,
        num_files,
        seeders,
        size,
        status,
        username,
        magnet,
    } = pb_t;

    Torrent {
        added,
        category,
        descr,
        download_count,
        id,
        info_hash,
        leechers,
        name,
        num_files,
        seeders,
        size,
        status,
        username,
        magnet,
    }
}

#[tauri::command]
async fn search(state: State<'_, App>, query: &str) -> Result<Vec<Torrent>, Error> {
    log::info!("searching: {query}");
    let torrents = state.client.search(query).await?;
    let torrents = torrents
        .into_iter()
        .map(pb_torrent_to_wire)
        .collect::<Vec<_>>();
    Ok(torrents)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    env_logger::builder().init();
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            #[cfg(debug_assertions)]
            {
                let window = app.get_webview_window("main").unwrap();
                window.open_devtools();
                // window.close_devtools();
            }
            app.manage(App {
                client: PirateClient::new(),
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![greet, search])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
