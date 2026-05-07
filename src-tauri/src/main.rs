// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::net::TcpStream;
use std::process::Command;
use std::time::{Duration, Instant};

const BACKEND_PORT: u16 = 3000;
const MAX_WAIT_SECS: u64 = 120;

fn start_backend() {
    std::thread::spawn(|| {
        let backend_dir = std::env::current_dir()
            .unwrap()
            .parent()
            .unwrap()
            .to_path_buf();
        let mut child = Command::new("cargo")
            .args(["run", "--bin", "quantpilot"])
            .current_dir(&backend_dir)
            .spawn()
            .expect("Failed to start backend");
        child.wait().ok();
    });
}

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
                    eprintln!("[tauri] Backend did not start within {}s, proceeding anyway", MAX_WAIT_SECS);
                    return;
                }
                std::thread::sleep(Duration::from_millis(500));
            }
        }
    }
}

fn main() {
    start_backend();
    eprintln!("[tauri] Waiting for backend (port {})...", BACKEND_PORT);
    wait_for_backend();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
