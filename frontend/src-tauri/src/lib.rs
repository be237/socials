#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                let backend_dir =
                    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../backend");
                let python = if std::process::Command::new("python3")
                    .arg("--version")
                    .status()
                    .is_ok()
                {
                    "python3"
                } else {
                    "python"
                };
                let result = std::process::Command::new(python)
                    .current_dir(&backend_dir)
                    .args(["-m", "app.main"])
                    .envs(std::env::vars())
                    .spawn();
                match result {
                    Ok(_) => {
                        use tauri::Emitter;
                        let _ = app_handle.emit("server-ready", ());
                    }
                    Err(error) => eprintln!("Impossible de lancer le backend Python: {error}"),
                }
            });
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running Tauri application");
}
