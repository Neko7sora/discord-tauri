#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::{webview::NewWindowResponse, Manager, WebviewUrl, WebviewWindowBuilder};
use url::Url;

const APP_TITLE: &str = "discord-tauri";
const DISCORD_URL: &str = "https://discord.com/login";
const DISCORD_SCHEME: &str = "https";
const DISCORD_HOST: &str = "discord.com";

fn is_allowed_in_webview(url: &Url) -> bool {
    url.scheme() == DISCORD_SCHEME && url.host_str() == Some(DISCORD_HOST)
}

fn is_http_or_https(url: &Url) -> bool {
    matches!(url.scheme(), "http" | "https")
}

fn open_external_in_browser(url: &Url) {
    if let Err(error) = open::that(url.as_str()) {
        eprintln!("failed to open external url in browser: {url} ({error})");
    }
}

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            let discord_url = Url::parse(DISCORD_URL).expect("discord login URL must be valid");
            let app_handle = app.handle().clone();

            WebviewWindowBuilder::new(app, "main", WebviewUrl::External(discord_url))
                .title(APP_TITLE)
                .decorations(false)
                .zoom_hotkeys_enabled(true)
                .inner_size(1320.0, 860.0)
                .min_inner_size(920.0, 640.0)
                .resizable(true)
                .on_document_title_changed(|window, title| {
                    let trimmed = title.trim();
                    let next_title = if trimmed.is_empty() {
                        APP_TITLE.to_string()
                    } else {
                        trimmed.to_string()
                    };

                    if let Err(error) = window.set_title(&next_title) {
                        eprintln!("failed to update window title: {error}");
                    }
                })
                .on_navigation(|url| {
                    if is_allowed_in_webview(url) {
                        true
                    } else if is_http_or_https(url) {
                        open_external_in_browser(url);
                        false
                    } else {
                        false
                    }
                })
                .on_new_window(move |url, _features| {
                    if is_allowed_in_webview(&url) {
                        // Deny the new window and redirect the main window instead.
                        // A window created via NewWindowResponse::Allow would lack
                        // the on_navigation / on_new_window handlers, so any page
                        // loaded in it could navigate freely and bypass the
                        // discord.com-only restriction.
                        if let Some(main_window) = app_handle.get_webview_window("main") {
                            let escaped = url.as_str().replace('\\', "\\\\").replace('"', "\\\"");
                            let js = format!("window.location.href = \"{escaped}\";");
                            if let Err(error) = main_window.eval(&js) {
                                eprintln!("failed to redirect main window: {error}");
                            }
                        }
                        NewWindowResponse::Deny
                    } else if is_http_or_https(&url) {
                        open_external_in_browser(&url);
                        NewWindowResponse::Deny
                    } else {
                        NewWindowResponse::Deny
                    }
                })
                .build()?;

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
