fn main() {
    tauri_build::try_build(tauri_build::Attributes::new().app_manifest(
        tauri_build::AppManifest::new().commands(&[
            "get_settings",
            "update_settings",
            "get_downloads",
            "cancel_download",
            "reveal_download",
            "clear_download_history",
            "clear_cache",
            "logout_and_clear_cookies",
            "client_event",
            "open_external",
            "download_url",
            "open_client_settings",
            "retry_connection",
            "check_for_update",
            "install_update",
        ]),
    ))
    .expect("failed to build Tauri application manifest");
}
