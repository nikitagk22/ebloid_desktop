#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use reqwest::header::{CONTENT_DISPOSITION, COOKIE, REFERER, USER_AGENT};
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    fs::{self, File},
    io::{Read, Write},
    path::{Path, PathBuf},
    process::Command,
    sync::{
        atomic::{AtomicBool, AtomicU64, Ordering},
        Arc, Mutex,
    },
    thread,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use tauri::{
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
    webview::{DownloadEvent, NewWindowResponse},
    AppHandle, Emitter, Manager, WebviewUrl, WebviewWindow, WebviewWindowBuilder, WindowEvent,
};
use tauri_plugin_autostart::ManagerExt as AutostartExt;
use tauri_plugin_deep_link::DeepLinkExt;
use tauri_plugin_notification::NotificationExt;
use tauri_plugin_updater::UpdaterExt;
use url::Url;

const HOME: &str = "https://eblo.id/";
const CLIENT_SCRIPT: &str = include_str!("../../ui/client.js");
const MAX_DOWNLOAD_HISTORY: usize = 200;
const MAX_SEEN_NOTIFICATIONS: usize = 500;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default)]
struct ClientSettings {
    notifications: bool,
    autostart: bool,
    minimize_to_tray: bool,
    zoom: f64,
    check_updates: bool,
    seen_notifications: Vec<String>,
}

impl Default for ClientSettings {
    fn default() -> Self {
        Self {
            notifications: true,
            autostart: false,
            minimize_to_tray: true,
            zoom: 1.0,
            check_updates: true,
            seen_notifications: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct DownloadItem {
    id: String,
    file_name: String,
    source_url: String,
    destination: String,
    status: String,
    downloaded_bytes: u64,
    total_bytes: Option<u64>,
    error: Option<String>,
    created_at: u64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SiteEvent {
    kind: String,
    unread: Option<u64>,
    notification_id: Option<String>,
    title: Option<String>,
    body: Option<String>,
    url: Option<String>,
    online: Option<bool>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct UpdateInfo {
    available: bool,
    current_version: String,
    version: Option<String>,
    notes: Option<String>,
}

struct AppState {
    settings: Mutex<ClientSettings>,
    downloads: Mutex<Vec<DownloadItem>>,
    cancellations: Mutex<HashMap<String, Arc<AtomicBool>>>,
    unread: AtomicU64,
    last_heartbeat: AtomicU64,
    online: AtomicBool,
    quitting: AtomicBool,
    download_sequence: AtomicU64,
}

fn unix_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn config_path(app: &AppHandle, file_name: &str) -> Option<PathBuf> {
    app.path()
        .app_config_dir()
        .ok()
        .map(|directory| directory.join(file_name))
}

fn read_json<T: for<'de> Deserialize<'de>>(path: Option<PathBuf>) -> Option<T> {
    let path = path?;
    let contents = fs::read_to_string(path).ok()?;
    serde_json::from_str(&contents).ok()
}

fn write_json<T: Serialize>(path: Option<PathBuf>, value: &T) -> Result<(), String> {
    let path = path.ok_or_else(|| "Не удалось определить каталог приложения".to_owned())?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    let temporary = path.with_extension("tmp");
    let data = serde_json::to_vec_pretty(value).map_err(|error| error.to_string())?;
    fs::write(&temporary, data).map_err(|error| error.to_string())?;
    fs::rename(temporary, path).map_err(|error| error.to_string())
}

fn save_settings(app: &AppHandle) -> Result<(), String> {
    let settings = app.state::<AppState>().settings.lock().unwrap().clone();
    write_json(config_path(app, "settings.json"), &settings)
}

fn save_downloads(app: &AppHandle) -> Result<(), String> {
    let downloads = app.state::<AppState>().downloads.lock().unwrap().clone();
    write_json(config_path(app, "downloads.json"), &downloads)
}

fn is_embedded_navigation(url: &Url) -> bool {
    is_ebloid_url(url) || matches!(url.scheme(), "about" | "data" | "blob")
}

fn is_system_link(url: &Url) -> bool {
    matches!(url.scheme(), "https" | "http" | "mailto" | "tel")
}

fn is_downloadable_url(url: &Url) -> bool {
    matches!(url.scheme(), "https" | "http")
}

fn suggested_filename(url: &Url) -> String {
    url.path_segments()
        .and_then(|mut segments| segments.rfind(|segment| !segment.is_empty()))
        .filter(|name| name.contains('.'))
        .unwrap_or("ebloid-download")
        .to_owned()
}

fn decode_percent_utf8(value: &str) -> Option<String> {
    let bytes = value.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'%' && index + 2 < bytes.len() {
            let hex = std::str::from_utf8(&bytes[index + 1..index + 3]).ok()?;
            decoded.push(u8::from_str_radix(hex, 16).ok()?);
            index += 3;
        } else {
            decoded.push(bytes[index]);
            index += 1;
        }
    }
    String::from_utf8(decoded).ok()
}

fn safe_download_name(value: &str) -> Option<String> {
    let name = value
        .trim()
        .trim_matches('"')
        .replace(['/', '\\', '\0'], "_");
    (!name.is_empty() && name != "." && name != "..").then_some(name)
}

fn content_disposition_filename(value: &str) -> Option<String> {
    let parameters = value.split(';').map(str::trim).collect::<Vec<_>>();
    if let Some(encoded) = parameters.iter().find_map(|parameter| {
        parameter
            .strip_prefix("filename*=")
            .or_else(|| parameter.strip_prefix("FILENAME*="))
    }) {
        let encoded = encoded
            .split_once("''")
            .map(|(_, value)| value)
            .unwrap_or(encoded);
        if let Some(name) = decode_percent_utf8(encoded).and_then(|name| safe_download_name(&name))
        {
            return Some(name);
        }
    }
    parameters.iter().find_map(|parameter| {
        parameter
            .strip_prefix("filename=")
            .or_else(|| parameter.strip_prefix("FILENAME="))
            .and_then(safe_download_name)
    })
}

fn remote_filename(url: &Url, cookies: &str) -> Option<String> {
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(15))
        .build()
        .ok()?;
    let mut request = client
        .head(url.clone())
        .header(
            USER_AGENT,
            concat!("Ebloid Desktop/", env!("CARGO_PKG_VERSION")),
        )
        .header(REFERER, HOME);
    if !cookies.is_empty() {
        request = request.header(COOKIE, cookies);
    }
    let response = request.send().ok()?.error_for_status().ok()?;
    response
        .headers()
        .get(CONTENT_DISPOSITION)?
        .to_str()
        .ok()
        .and_then(content_disposition_filename)
}

fn available_download_path(directory: &Path, file_name: &str) -> PathBuf {
    let initial = directory.join(file_name);
    if !initial.exists() {
        return initial;
    }
    let path = Path::new(file_name);
    let stem = path
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or("file");
    let extension = path.extension().and_then(|value| value.to_str());
    for index in 1..10_000 {
        let candidate_name = match extension {
            Some(extension) => format!("{stem} ({index}).{extension}"),
            None => format!("{stem} ({index})"),
        };
        let candidate = directory.join(candidate_name);
        if !candidate.exists() {
            return candidate;
        }
    }
    directory.join(format!("{}-{file_name}", unix_seconds()))
}

fn is_ebloid_url(url: &Url) -> bool {
    matches!(url.scheme(), "https" | "http")
        && url
            .host_str()
            .is_some_and(|host| host == "eblo.id" || host.ends_with(".eblo.id"))
}

fn destination_from_deep_link(url: &Url) -> Option<Url> {
    if url.scheme() != "ebloid" || url.host_str() == Some("share") {
        return None;
    }

    let destination = if url.host_str() == Some("open") {
        url.query_pairs()
            .find(|(key, _)| key == "url")
            .and_then(|(_, value)| Url::parse(&value).ok())
    } else {
        let host_path = url.host_str().unwrap_or_default();
        let query = url
            .query()
            .map(|value| format!("?{value}"))
            .unwrap_or_default();
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
    if url.scheme() == "ebloid" && matches!(url.host_str(), Some("settings") | Some("downloads")) {
        let section = if url.host_str() == Some("downloads") {
            "downloads"
        } else {
            "general"
        };
        let _ = open_settings_window(app, Some(section));
        return;
    }
    let Some(destination) = destination_from_deep_link(url) else {
        return;
    };
    focus_main_window(app);
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.navigate(destination);
    }
}

fn open_with_system(target: &str) -> Result<(), String> {
    let mut command = if cfg!(target_os = "macos") {
        let mut command = Command::new("open");
        command.arg(target);
        command
    } else if cfg!(target_os = "windows") {
        let mut command = Command::new("rundll32.exe");
        command.arg("url.dll,FileProtocolHandler").arg(target);
        command
    } else {
        let mut command = Command::new("xdg-open");
        command.arg(target);
        command
    };
    command
        .spawn()
        .map(|_| ())
        .map_err(|error| error.to_string())
}

fn reveal_in_file_manager(path: &Path) -> Result<(), String> {
    let mut command = if cfg!(target_os = "macos") {
        let mut command = Command::new("open");
        command.arg("-R").arg(path);
        command
    } else if cfg!(target_os = "windows") {
        let mut command = Command::new("explorer.exe");
        command.arg(format!("/select,{}", path.display()));
        command
    } else {
        let mut command = Command::new("xdg-open");
        command.arg(path.parent().unwrap_or(path));
        command
    };
    command
        .spawn()
        .map(|_| ())
        .map_err(|error| error.to_string())
}

fn tray_menu(app: &AppHandle, unread: u64) -> tauri::Result<Menu<tauri::Wry>> {
    let unread_label = if unread == 0 {
        "Нет новых уведомлений".to_owned()
    } else {
        format!("Непрочитанных: {unread}")
    };
    let unread_item = MenuItem::with_id(app, "unread", unread_label, false, None::<&str>)?;
    let show = MenuItem::with_id(app, "show", "Открыть Ebloid", true, None::<&str>)?;
    let settings = MenuItem::with_id(app, "settings", "Настройки клиента", true, None::<&str>)?;
    let downloads = MenuItem::with_id(app, "downloads", "Загрузки", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Выйти", true, None::<&str>)?;
    Menu::with_items(app, &[&unread_item, &show, &settings, &downloads, &quit])
}

fn update_unread_ui(app: &AppHandle, unread: u64) {
    app.state::<AppState>()
        .unread
        .store(unread, Ordering::Relaxed);
    if let Some(window) = app.get_webview_window("main") {
        let title = if unread == 0 {
            "Ebloid".to_owned()
        } else {
            format!("({unread}) Ebloid")
        };
        let _ = window.set_title(&title);
        let _ = window.set_badge_count((unread > 0).then_some(unread as i64));
        #[cfg(target_os = "windows")]
        {
            let overlay = badge_icon(unread);
            let _ = window.set_overlay_icon(overlay);
        }
    }
    if let Some(tray) = app.tray_by_id("main-tray") {
        let tooltip = if unread == 0 {
            "Ebloid".to_owned()
        } else {
            format!("Ebloid — {unread} непрочитанных")
        };
        let _ = tray.set_tooltip(Some(tooltip));
        if let Ok(menu) = tray_menu(app, unread) {
            let _ = tray.set_menu(Some(menu));
        }
    }
    let _ = app.emit("unread-changed", unread);
}

#[cfg(target_os = "windows")]
fn badge_icon(unread: u64) -> Option<tauri::image::Image<'static>> {
    if unread == 0 {
        return None;
    }
    let bytes: &'static [u8] = match unread {
        1 => include_bytes!("../icons/badges/1.png"),
        2 => include_bytes!("../icons/badges/2.png"),
        3 => include_bytes!("../icons/badges/3.png"),
        4 => include_bytes!("../icons/badges/4.png"),
        5 => include_bytes!("../icons/badges/5.png"),
        6 => include_bytes!("../icons/badges/6.png"),
        7 => include_bytes!("../icons/badges/7.png"),
        8 => include_bytes!("../icons/badges/8.png"),
        9 => include_bytes!("../icons/badges/9.png"),
        _ => include_bytes!("../icons/badges/9plus.png"),
    };
    tauri::image::Image::from_bytes(bytes).ok()
}

fn open_settings_window(app: &AppHandle, section: Option<&str>) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("settings") {
        window.show().map_err(|error| error.to_string())?;
        window.set_focus().map_err(|error| error.to_string())?;
        if let Some(section) = section {
            let script = format!(
                "location.hash={0};window.dispatchEvent(new CustomEvent('ebloid-section', {{detail:{0}}}));",
                serde_json::to_string(section).unwrap()
            );
            let _ = window.eval(script);
        }
        return Ok(());
    }

    let page = format!("settings.html#{}", section.unwrap_or("general"));
    WebviewWindowBuilder::new(app, "settings", WebviewUrl::App(page.into()))
        .title("Настройки Ebloid")
        .inner_size(880.0, 720.0)
        .min_inner_size(720.0, 580.0)
        .resizable(true)
        .build()
        .map_err(|error| error.to_string())?;
    Ok(())
}

fn cookie_header(window: &WebviewWindow, url: &Url) -> String {
    window
        .cookies_for_url(url.clone())
        .unwrap_or_default()
        .into_iter()
        .map(|cookie| format!("{}={}", cookie.name(), cookie.value()))
        .collect::<Vec<_>>()
        .join("; ")
}

fn emit_downloads(app: &AppHandle) {
    let downloads = app.state::<AppState>().downloads.lock().unwrap().clone();
    let latest = serde_json::to_string(&downloads.first()).unwrap_or_else(|_| "null".to_owned());
    let _ = app.emit("downloads-changed", downloads);
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.eval(format!("window.__ebloidClientDownload?.({latest});"));
    }
}

fn update_download<F: FnOnce(&mut DownloadItem)>(app: &AppHandle, id: &str, update: F) {
    {
        let state = app.state::<AppState>();
        let mut downloads = state.downloads.lock().unwrap();
        if let Some(item) = downloads.iter_mut().find(|item| item.id == id) {
            update(item);
        }
    }
    let _ = save_downloads(app);
    emit_downloads(app);
}

fn start_download(app: &AppHandle, url: Url) -> Result<String, String> {
    if !is_downloadable_url(&url) {
        return Err("Разрешены только HTTP(S)-ссылки".to_owned());
    }
    let cookies = app
        .get_webview_window("main")
        .map(|window| cookie_header(&window, &url))
        .unwrap_or_default();
    let file_name = remote_filename(&url, &cookies).unwrap_or_else(|| suggested_filename(&url));
    let download_directory = app
        .path()
        .download_dir()
        .map_err(|error| error.to_string())?;
    fs::create_dir_all(&download_directory).map_err(|error| error.to_string())?;
    let destination = available_download_path(&download_directory, &file_name);

    let state = app.state::<AppState>();
    let sequence = state.download_sequence.fetch_add(1, Ordering::Relaxed);
    let id = format!("{}-{sequence}", unix_seconds());
    let cancel = Arc::new(AtomicBool::new(false));
    state
        .cancellations
        .lock()
        .unwrap()
        .insert(id.clone(), cancel.clone());
    {
        let mut downloads = state.downloads.lock().unwrap();
        downloads.insert(
            0,
            DownloadItem {
                id: id.clone(),
                file_name,
                source_url: url.to_string(),
                destination: destination.to_string_lossy().into_owned(),
                status: "queued".to_owned(),
                downloaded_bytes: 0,
                total_bytes: None,
                error: None,
                created_at: unix_seconds(),
            },
        );
        downloads.truncate(MAX_DOWNLOAD_HISTORY);
    }
    let _ = save_downloads(app);
    emit_downloads(app);

    let app_handle = app.clone();
    let download_id = id.clone();
    thread::spawn(move || {
        let result = download_worker(
            &app_handle,
            &download_id,
            &url,
            &destination,
            &cookies,
            &cancel,
        );
        if let Err(error) = result {
            let _ = fs::remove_file(&destination);
            let status = if cancel.load(Ordering::Relaxed) {
                "cancelled"
            } else {
                "failed"
            };
            update_download(&app_handle, &download_id, |item| {
                item.status = status.to_owned();
                item.error = (status == "failed").then_some(error);
            });
        }
        app_handle
            .state::<AppState>()
            .cancellations
            .lock()
            .unwrap()
            .remove(&download_id);
    });
    Ok(id)
}

fn download_worker(
    app: &AppHandle,
    id: &str,
    url: &Url,
    destination: &Path,
    cookies: &str,
    cancel: &AtomicBool,
) -> Result<(), String> {
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(60 * 60))
        .build()
        .map_err(|error| error.to_string())?;
    let mut request = client
        .get(url.clone())
        .header(
            USER_AGENT,
            concat!("Ebloid Desktop/", env!("CARGO_PKG_VERSION")),
        )
        .header(REFERER, HOME);
    if !cookies.is_empty() {
        request = request.header(COOKIE, cookies);
    }
    let mut response = request
        .send()
        .and_then(reqwest::blocking::Response::error_for_status)
        .map_err(|error| error.to_string())?;
    let total = response.content_length();
    update_download(app, id, |item| {
        item.status = "downloading".to_owned();
        item.total_bytes = total;
    });

    let mut file = File::create(destination).map_err(|error| error.to_string())?;
    let mut buffer = vec![0_u8; 128 * 1024];
    let mut downloaded = 0_u64;
    let mut last_emit = 0_u64;
    loop {
        if cancel.load(Ordering::Relaxed) {
            return Err("Загрузка отменена".to_owned());
        }
        let read = response
            .read(&mut buffer)
            .map_err(|error| error.to_string())?;
        if read == 0 {
            break;
        }
        file.write_all(&buffer[..read])
            .map_err(|error| error.to_string())?;
        downloaded += read as u64;
        if downloaded.saturating_sub(last_emit) >= 256 * 1024 || total == Some(downloaded) {
            last_emit = downloaded;
            update_download(app, id, |item| item.downloaded_bytes = downloaded);
        }
    }
    file.flush().map_err(|error| error.to_string())?;
    update_download(app, id, |item| {
        item.status = "completed".to_owned();
        item.downloaded_bytes = downloaded;
        item.error = None;
    });
    Ok(())
}

#[tauri::command]
fn get_settings(app: AppHandle) -> ClientSettings {
    app.state::<AppState>().settings.lock().unwrap().clone()
}

#[tauri::command]
fn update_settings(app: AppHandle, mut settings: ClientSettings) -> Result<ClientSettings, String> {
    settings.zoom = settings.zoom.clamp(0.75, 1.5);
    if settings.seen_notifications.len() > MAX_SEEN_NOTIFICATIONS {
        let keep_from = settings.seen_notifications.len() - MAX_SEEN_NOTIFICATIONS;
        settings.seen_notifications.drain(..keep_from);
    }
    if settings.autostart {
        app.autolaunch()
            .enable()
            .map_err(|error| error.to_string())?;
    } else {
        app.autolaunch()
            .disable()
            .map_err(|error| error.to_string())?;
    }
    if let Some(window) = app.get_webview_window("main") {
        window
            .set_zoom(settings.zoom)
            .map_err(|error| error.to_string())?;
    }
    *app.state::<AppState>().settings.lock().unwrap() = settings.clone();
    save_settings(&app)?;
    Ok(settings)
}

#[tauri::command]
fn get_downloads(app: AppHandle) -> Vec<DownloadItem> {
    app.state::<AppState>().downloads.lock().unwrap().clone()
}

#[tauri::command]
fn cancel_download(app: AppHandle, id: String) -> Result<(), String> {
    let state = app.state::<AppState>();
    let cancellations = state.cancellations.lock().unwrap();
    let cancel = cancellations
        .get(&id)
        .ok_or_else(|| "Эта загрузка уже завершена".to_owned())?;
    cancel.store(true, Ordering::Relaxed);
    Ok(())
}

#[tauri::command]
fn reveal_download(app: AppHandle, id: String) -> Result<(), String> {
    let path = app
        .state::<AppState>()
        .downloads
        .lock()
        .unwrap()
        .iter()
        .find(|item| item.id == id)
        .map(|item| PathBuf::from(&item.destination))
        .ok_or_else(|| "Загрузка не найдена".to_owned())?;
    reveal_in_file_manager(&path)
}

#[tauri::command]
fn clear_download_history(app: AppHandle) -> Result<(), String> {
    app.state::<AppState>()
        .downloads
        .lock()
        .unwrap()
        .retain(|item| matches!(item.status.as_str(), "queued" | "downloading"));
    save_downloads(&app)?;
    emit_downloads(&app);
    Ok(())
}

#[tauri::command]
fn clear_cache(app: AppHandle) -> Result<(), String> {
    let window = app
        .get_webview_window("main")
        .ok_or_else(|| "Главное окно не найдено".to_owned())?;
    let home = Url::parse(HOME).unwrap();
    let cookies = window.cookies_for_url(home).unwrap_or_default();
    window
        .clear_all_browsing_data()
        .map_err(|error| error.to_string())?;
    for cookie in cookies {
        let _ = window.set_cookie(cookie);
    }
    window.reload().map_err(|error| error.to_string())
}

#[tauri::command]
fn logout_and_clear_cookies(app: AppHandle) -> Result<(), String> {
    let window = app
        .get_webview_window("main")
        .ok_or_else(|| "Главное окно не найдено".to_owned())?;
    let home = Url::parse(HOME).unwrap();
    for cookie in window.cookies_for_url(home).unwrap_or_default() {
        let _ = window.delete_cookie(cookie);
    }
    let _ = window.eval(
        "try{localStorage.clear();sessionStorage.clear();caches.keys().then(k=>k.forEach(x=>caches.delete(x)))}catch(_){}",
    );
    window
        .navigate(Url::parse(HOME).unwrap())
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn open_external(url: String) -> Result<(), String> {
    let parsed = Url::parse(&url).map_err(|error| error.to_string())?;
    if !is_system_link(&parsed) {
        return Err("Эту ссылку нельзя открыть во внешнем приложении".to_owned());
    }
    open_with_system(parsed.as_str())
}

#[tauri::command]
fn download_url(app: AppHandle, url: String) -> Result<String, String> {
    let parsed = Url::parse(&url).map_err(|error| error.to_string())?;
    start_download(&app, parsed)
}

#[tauri::command]
fn open_client_settings(app: AppHandle, section: Option<String>) -> Result<(), String> {
    open_settings_window(&app, section.as_deref())
}

#[tauri::command]
fn retry_connection(app: AppHandle) -> Result<(), String> {
    app.state::<AppState>()
        .last_heartbeat
        .store(unix_seconds(), Ordering::Relaxed);
    app.get_webview_window("main")
        .ok_or_else(|| "Главное окно не найдено".to_owned())?
        .reload()
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn client_event(app: AppHandle, event: SiteEvent) -> Result<(), String> {
    app.state::<AppState>()
        .last_heartbeat
        .store(unix_seconds(), Ordering::Relaxed);
    if let Some(online) = event.online {
        app.state::<AppState>()
            .online
            .store(online, Ordering::Relaxed);
    }
    if let Some(unread) = event.unread {
        update_unread_ui(&app, unread);
    }
    if !matches!(event.kind.as_str(), "notification" | "seedNotification") {
        return Ok(());
    }

    let Some(notification_id) = event.notification_id else {
        return Ok(());
    };
    let state = app.state::<AppState>();
    let mut settings = state.settings.lock().unwrap();
    let seen: HashSet<&str> = settings
        .seen_notifications
        .iter()
        .map(String::as_str)
        .collect();
    let is_new = !seen.contains(notification_id.as_str());
    if is_new {
        settings.seen_notifications.push(notification_id);
        if settings.seen_notifications.len() > MAX_SEEN_NOTIFICATIONS {
            settings.seen_notifications.remove(0);
        }
    }
    let notifications_enabled = settings.notifications;
    drop(settings);
    let _ = save_settings(&app);

    if is_new && notifications_enabled && event.kind == "notification" {
        let title = event
            .title
            .unwrap_or_else(|| "Новое уведомление".to_owned());
        let body = event.body.unwrap_or_default();
        let mut notification = app.notification().builder().title(title).body(body);
        if let Some(url) = event.url {
            notification = notification.extra("url", url);
        }
        let _ = notification.show();
    }
    Ok(())
}

#[tauri::command]
async fn check_for_update(app: AppHandle) -> Result<UpdateInfo, String> {
    let current_version = app.package_info().version.to_string();
    let update = app
        .updater()
        .map_err(|error| error.to_string())?
        .check()
        .await
        .map_err(|error| error.to_string())?;
    Ok(match update {
        Some(update) => UpdateInfo {
            available: true,
            current_version,
            version: Some(update.version),
            notes: update.body,
        },
        None => UpdateInfo {
            available: false,
            current_version,
            version: None,
            notes: None,
        },
    })
}

#[tauri::command]
async fn install_update(app: AppHandle) -> Result<(), String> {
    let update = app
        .updater()
        .map_err(|error| error.to_string())?
        .check()
        .await
        .map_err(|error| error.to_string())?
        .ok_or_else(|| "Новых версий нет".to_owned())?;
    let progress_app = app.clone();
    let mut downloaded_bytes = 0_usize;
    update
        .download_and_install(
            move |chunk, total| {
                downloaded_bytes = downloaded_bytes.saturating_add(chunk);
                let _ = progress_app.emit(
                    "update-progress",
                    serde_json::json!({"chunkBytes": downloaded_bytes, "totalBytes": total}),
                );
            },
            {
                let app = app.clone();
                move || {
                    let _ = app.emit("update-downloaded", ());
                }
            },
        )
        .await
        .map_err(|error| error.to_string())?;
    app.restart();
}

fn create_main_window(app: &AppHandle) -> tauri::Result<WebviewWindow> {
    let home = Url::parse(HOME).expect("the Ebloid URL is valid");
    let download_app = app.clone();
    let navigation_app = app.clone();
    let new_window_app = app.clone();
    let window = WebviewWindowBuilder::new(app, "main", WebviewUrl::External(home))
        .title("Ebloid")
        .inner_size(1440.0, 920.0)
        .min_inner_size(900.0, 650.0)
        .resizable(true)
        .incognito(false)
        .enable_clipboard_access()
        .disable_drag_drop_handler()
        .zoom_hotkeys_enabled(true)
        .initialization_script(CLIENT_SCRIPT)
        .on_navigation(move |url| {
            if is_embedded_navigation(url) {
                return true;
            }

            if is_system_link(url) {
                let _ = open_with_system(url.as_str());
                focus_main_window(&navigation_app);
            }
            false
        })
        .on_new_window(move |url, _features| {
            if is_ebloid_url(&url) {
                if let Some(window) = new_window_app.get_webview_window("main") {
                    let _ = window.navigate(url);
                    let _ = window.set_focus();
                }
            } else if is_system_link(&url) {
                let _ = open_with_system(url.as_str());
            }
            NewWindowResponse::Deny
        })
        .on_download(move |_webview, event| match event {
            DownloadEvent::Requested { url, .. } => {
                let app = download_app.clone();
                thread::spawn(move || {
                    let _ = start_download(&app, url);
                });
                false
            }
            DownloadEvent::Finished { .. } => true,
            _ => true,
        })
        .build()?;

    let close_app = app.clone();
    window.on_window_event(move |event| {
        if let WindowEvent::CloseRequested { api, .. } = event {
            let state = close_app.state::<AppState>();
            if !state.quitting.load(Ordering::Relaxed)
                && state.settings.lock().unwrap().minimize_to_tray
            {
                api.prevent_close();
                if let Some(window) = close_app.get_webview_window("main") {
                    let _ = window.hide();
                }
            }
        }
    });
    Ok(window)
}

fn start_health_monitor(app: AppHandle) {
    thread::spawn(move || {
        let client = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(8))
            .build()
            .ok();
        loop {
            thread::sleep(Duration::from_secs(15));
            let Some(window) = app.get_webview_window("main") else {
                break;
            };
            let reachable = client
                .as_ref()
                .and_then(|client| client.get(HOME).send().ok())
                .map(|response| {
                    response.status().is_success() || response.status().is_redirection()
                })
                .unwrap_or(false);
            app.state::<AppState>()
                .online
                .store(reachable, Ordering::Relaxed);
            let script = format!("window.__ebloidClientSetOnline?.({reachable});");
            let _ = window.eval(script);

            let heartbeat = app
                .state::<AppState>()
                .last_heartbeat
                .load(Ordering::Relaxed);
            if reachable && unix_seconds().saturating_sub(heartbeat) > 45 {
                app.state::<AppState>()
                    .last_heartbeat
                    .store(unix_seconds(), Ordering::Relaxed);
                let _ = window.reload();
            }
        }
    });
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, argv, _cwd| {
            focus_main_window(app);
            for argument in &argv {
                if let Ok(url) = Url::parse(argument) {
                    open_deep_link(app, &url);
                }
            }
        }))
        .plugin(tauri_plugin_deep_link::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .invoke_handler(tauri::generate_handler![
            get_settings,
            update_settings,
            get_downloads,
            cancel_download,
            reveal_download,
            clear_download_history,
            clear_cache,
            logout_and_clear_cookies,
            open_external,
            download_url,
            open_client_settings,
            retry_connection,
            client_event,
            check_for_update,
            install_update
        ])
        .setup(|app| {
            app.handle().plugin(tauri_plugin_autostart::init(
                tauri_plugin_autostart::MacosLauncher::LaunchAgent,
                None,
            ))?;

            let settings: ClientSettings =
                read_json(config_path(app.handle(), "settings.json")).unwrap_or_default();
            let downloads: Vec<DownloadItem> =
                read_json(config_path(app.handle(), "downloads.json")).unwrap_or_default();
            app.manage(AppState {
                settings: Mutex::new(settings.clone()),
                downloads: Mutex::new(downloads),
                cancellations: Mutex::new(HashMap::new()),
                unread: AtomicU64::new(0),
                last_heartbeat: AtomicU64::new(unix_seconds()),
                online: AtomicBool::new(true),
                quitting: AtomicBool::new(false),
                download_sequence: AtomicU64::new(0),
            });

            let window = create_main_window(app.handle())?;
            let _ = window.set_zoom(settings.zoom);
            let menu = tray_menu(app.handle(), 0)?;
            let tray_app = app.handle().clone();
            TrayIconBuilder::with_id("main-tray")
                .icon(app.default_window_icon().unwrap().clone())
                .tooltip("Ebloid")
                .menu(&menu)
                .on_menu_event(move |app, event| match event.id.as_ref() {
                    "show" => focus_main_window(app),
                    "settings" => {
                        let _ = open_settings_window(app, Some("general"));
                    }
                    "downloads" => {
                        let _ = open_settings_window(app, Some("downloads"));
                    }
                    "quit" => {
                        app.state::<AppState>()
                            .quitting
                            .store(true, Ordering::Relaxed);
                        app.exit(0);
                    }
                    _ => {}
                })
                .build(&tray_app)?;

            let app_handle = app.handle().clone();
            if let Some(urls) = app.deep_link().get_current()? {
                for url in urls {
                    open_deep_link(&app_handle, &url);
                }
            }
            app.deep_link().on_open_url(move |event| {
                for url in event.urls() {
                    open_deep_link(&app_handle, &url);
                }
            });

            start_health_monitor(app.handle().clone());

            if settings.check_updates {
                let update_app = app.handle().clone();
                tauri::async_runtime::spawn(async move {
                    let Ok(updater) = update_app.updater() else {
                        return;
                    };
                    let Ok(Some(update)) = updater.check().await else {
                        return;
                    };
                    let info = UpdateInfo {
                        available: true,
                        current_version: update.current_version,
                        version: Some(update.version.clone()),
                        notes: update.body.clone(),
                    };
                    let _ = update_app.emit("update-available", &info);
                    let _ = update_app
                        .notification()
                        .builder()
                        .title(format!("Доступен Ebloid {}", update.version))
                        .body("Откройте настройки клиента, чтобы установить обновление.")
                        .show();
                });
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("failed to start Ebloid");
}

#[cfg(test)]
mod tests {
    use super::{content_disposition_filename, is_ebloid_url, is_embedded_navigation};
    use url::Url;

    #[test]
    fn reads_utf8_download_filename() {
        let header = "attachment; filename=\"___ smeshno.opt.mp4\"; filename*=UTF-8''%D0%98%D0%BC%D1%8F%20smeshno.opt.mp4";
        assert_eq!(
            content_disposition_filename(header).as_deref(),
            Some("Имя smeshno.opt.mp4")
        );
    }

    #[test]
    fn removes_path_separators_from_download_filename() {
        assert_eq!(
            content_disposition_filename("attachment; filename=\"../../video.mp4\"").as_deref(),
            Some(".._.._video.mp4")
        );
    }

    #[test]
    fn keeps_only_ebloid_pages_inside_the_webview() {
        assert!(is_ebloid_url(&Url::parse("https://eblo.id/video").unwrap()));
        assert!(is_ebloid_url(
            &Url::parse("https://static.eblo.id/player").unwrap()
        ));
        assert!(!is_ebloid_url(
            &Url::parse("https://youtube.com/watch?v=1").unwrap()
        ));
        assert!(!is_ebloid_url(
            &Url::parse("https://eblo.id.example.com/").unwrap()
        ));
    }

    #[test]
    fn allows_internal_webview_schemes_without_opening_a_browser() {
        assert!(is_embedded_navigation(&Url::parse("about:blank").unwrap()));
        assert!(is_embedded_navigation(
            &Url::parse("blob:https://eblo.id/9f0c").unwrap()
        ));
    }
}
