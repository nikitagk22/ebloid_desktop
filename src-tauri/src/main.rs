#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::{
    webview::{DownloadEvent, NewWindowResponse},
    Manager, WebviewUrl, WebviewWindowBuilder,
};
use url::Url;

const HOME: &str = "https://eblo.id/";

/// The WebView may navigate to an OAuth provider and back, but never to a
/// custom protocol. This keeps `tg://`, `mailto:` and similar links from
/// unexpectedly taking over the desktop shell while allowing normal HTTPS
/// redirects, CDN pages and download endpoints used by the site.
fn is_safe_navigation(url: &Url) -> bool {
    matches!(url.scheme(), "https" | "http" | "about" | "data" | "blob")
}

fn suggested_filename(url: &Url) -> String {
    url.path_segments()
        .and_then(|segments| segments.filter(|segment| !segment.is_empty()).last())
        .filter(|name| name.contains('.'))
        .unwrap_or("ebloid-download")
        .to_owned()
}

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            let app_handle = app.handle().clone();
            let home = Url::parse(HOME).expect("the Ebloid URL is valid");

            WebviewWindowBuilder::new(&app_handle, "main", WebviewUrl::External(home))
                .title("Ebloid")
                .inner_size(1440.0, 920.0)
                .min_inner_size(900.0, 650.0)
                .resizable(true)
                // A normal, persistent profile is intentional: WebView keeps
                // HTTP cache, cookies, IndexedDB and localStorage between runs.
                // Cache-control headers from eblo.id remain authoritative.
                .incognito(false)
                .on_navigation(is_safe_navigation)
                // OAuth providers often use `window.open`. Tauri's default
                // implementation creates a native child window while retaining
                // the same WebView profile (and therefore the OAuth session).
                .on_new_window(|url, _features| {
                    if is_safe_navigation(&url) {
                        NewWindowResponse::Allow
                    } else {
                        NewWindowResponse::Deny
                    }
                })
                // Do not silently put files into Downloads. Each download asks
                // the user for a location using the OS-native save dialog.
                .on_download(|_window, event| match event {
                    DownloadEvent::Requested { url, destination } => {
                        let selected_path = rfd::FileDialog::new()
                            .set_title("Сохранить файл")
                            .set_file_name(suggested_filename(&url))
                            .save_file();

                        if let Some(path) = selected_path {
                            *destination = path;
                            true
                        } else {
                            false
                        }
                    }
                    DownloadEvent::Finished { .. } => true,
                    _ => true,
                })
                .build()?;

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("failed to start Ebloid");
}
