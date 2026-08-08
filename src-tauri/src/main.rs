#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::{
    webview::{DownloadEvent, NewWindowResponse},
    AppHandle, Manager, WebviewUrl, WebviewWindowBuilder,
};
use tauri_plugin_deep_link::DeepLinkExt;
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

fn is_ebloid_url(url: &Url) -> bool {
    matches!(url.scheme(), "https" | "http")
        && matches!(url.host_str(), Some("eblo.id") | Some("www.eblo.id"))
}

/// Converts only the app's own custom links to an eblo.id page. Accepting an
/// arbitrary destination here would let a malicious `ebloid://` URL navigate
/// an already authenticated WebView to a phishing site.
fn destination_from_deep_link(url: &Url) -> Option<Url> {
    if url.scheme() != "ebloid" {
        return None;
    }

    let destination = if url.host_str() == Some("open") {
        url.query_pairs()
            .find(|(key, _)| key == "url")
            .and_then(|(_, value)| Url::parse(&value).ok())
    } else {
        let host_path = url.host_str().unwrap_or_default();
        let query = url.query().map(|value| format!("?{value}")).unwrap_or_default();
        Url::parse(&format!("{HOME}{host_path}{}{}", url.path(), query)).ok()
    };

    destination.filter(is_ebloid_url)
}

fn focus_main_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
    }
}

fn open_deep_link(app: &AppHandle, url: &Url) {
    let Some(destination) = destination_from_deep_link(url) else {
        return;
    };

    focus_main_window(app);
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.navigate(destination);
    }
}

fn main() {
    tauri::Builder::default()
        // A second launch focuses the existing window instead of creating a
        // separate WebView profile and losing the deep-link destination.
        .plugin(tauri_plugin_single_instance::init(|app, _argv, _cwd| {
            focus_main_window(app);
        }))
        .plugin(tauri_plugin_deep_link::init())
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

            let app_handle = app.handle().clone();
            if let Some(urls) = app.deep_link().get_current()? {
                for url in urls {
                    open_deep_link(&app_handle, &url);
                }
            }
            app.deep_link().on_open_url(move |event| {
                for url in event.urls() {
                    open_deep_link(&app_handle, url);
                }
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("failed to start Ebloid");
}
