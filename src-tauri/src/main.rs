// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::net::TcpStream;
use std::time::{Duration, Instant};
use tauri::Manager;

const BACKEND_PORT: u16 = 3000;
const MAX_WAIT_SECS: u64 = 30;

fn wait_for_backend() {
    let start = Instant::now();
    loop {
        match TcpStream::connect_timeout(
            &format!("127.0.0.1:{}", BACKEND_PORT).parse().unwrap(),
            Duration::from_secs(1),
        ) {
            Ok(_) => {
                eprintln!("[tauri] Backend ready after {}s", start.elapsed().as_secs());
                return;
            }
            Err(_) => {
                if start.elapsed().as_secs() > MAX_WAIT_SECS {
                    eprintln!(
                        "[tauri] Backend not detected within {}s, proceeding anyway",
                        MAX_WAIT_SECS
                    );
                    return;
                }
                std::thread::sleep(Duration::from_millis(1000));
            }
        }
    }
}

fn main() {
    eprintln!("[tauri] Waiting for backend (port {})...", BACKEND_PORT);
    wait_for_backend();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            #[cfg(debug_assertions)]
            if let Some(window) = app.webview_windows().values().next() {
                window.open_devtools();
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
