#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{collections::HashMap, sync::Mutex};

use base64::{engine::general_purpose::STANDARD, Engine as _};
use reqwest::header::CONTENT_TYPE;
use tauri::{
    image::Image,
    webview::{NewWindowResponse, PageLoadEvent},
    Manager, State, WebviewUrl, WebviewWindow, WebviewWindowBuilder,
};
use url::Url;

const APP_TITLE: &str = "discord-tauri";
const DISCORD_URL: &str = "https://discord.com/login";
const DISCORD_SCHEME: &str = "https";
const DISCORD_HOST: &str = "discord.com";
const FAVICON_OBSERVER_SCRIPT: &str = r#"
(() => {
  if (window.__discordTauriFaviconObserverInstalled) {
    return;
  }

  Object.defineProperty(window, "__discordTauriFaviconObserverInstalled", {
    value: true,
    configurable: false,
    enumerable: false,
    writable: false
  });

  const invoke = window.__TAURI_INTERNALS__?.invoke;
  if (typeof invoke !== "function") {
    return;
  }

  let lastHref = "";

  const normalizeHref = (value) => {
    if (!value) {
      return "";
    }

    try {
      return new URL(value, window.location.href).href;
    } catch {
      return "";
    }
  };

  const resolveFaviconHref = () => {
    const links = Array.from(
      document.querySelectorAll("link[rel~='icon'], link[rel='shortcut icon'], link[rel='apple-touch-icon']")
    );

    for (const link of links) {
      const href = normalizeHref(link.getAttribute("href"));
      if (href) {
        return href;
      }
    }

    return normalizeHref("/favicon.ico");
  };

  const reportFavicon = () => {
    const href = resolveFaviconHref();
    if (!href || href === lastHref) {
      return;
    }

    lastHref = href;
    invoke("update_favicon", { href }).catch(() => {});
  };

  const observerTarget = document.head || document.documentElement;
  if (observerTarget) {
    new MutationObserver(() => queueMicrotask(reportFavicon)).observe(observerTarget, {
      subtree: true,
      childList: true,
      attributes: true,
      attributeFilter: ["href", "rel"]
    });
  }

  window.addEventListener("focus", reportFavicon);
  document.addEventListener("readystatechange", reportFavicon);
  reportFavicon();
})();
"#;

#[derive(Default)]
struct FaviconState {
    latest_by_window: Mutex<HashMap<String, String>>,
}

#[tauri::command]
fn update_favicon(
    app: tauri::AppHandle,
    window: WebviewWindow,
    state: State<'_, FaviconState>,
    href: String,
) {
    let window_label = window.label().to_string();

    {
        let mut latest_by_window = state.latest_by_window.lock().unwrap();
        if latest_by_window.get(&window_label) == Some(&href) {
            return;
        }
        latest_by_window.insert(window_label.clone(), href.clone());
    }

    std::thread::spawn(move || {
        let Some(icon) = load_favicon_image(&href) else {
            return;
        };

        let current_href = app
            .state::<FaviconState>()
            .latest_by_window
            .lock()
            .unwrap()
            .get(&window_label)
            .cloned();

        if current_href.as_deref() != Some(href.as_str()) {
            return;
        }

        let app_handle = app.clone();
        let _ = app.run_on_main_thread(move || {
            if let Some(window) = app_handle.get_webview_window(&window_label) {
                if let Err(error) = window.set_icon(icon) {
                    eprintln!("failed to update window icon: {error}");
                }
            }
        });
    });
}

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

fn load_favicon_image(href: &str) -> Option<Image<'static>> {
    let bytes = if href.starts_with("data:") {
        decode_data_url(href)?
    } else {
        fetch_remote_bytes(href)?
    };

    Image::from_bytes(&bytes)
        .map(Image::to_owned)
        .map_err(|error| {
            eprintln!("failed to decode favicon image: {href} ({error})");
            error
        })
        .ok()
}

fn fetch_remote_bytes(href: &str) -> Option<Vec<u8>> {
    let response = reqwest::blocking::Client::builder()
        .redirect(reqwest::redirect::Policy::limited(10))
        .build()
        .ok()?
        .get(href)
        .header(
            reqwest::header::USER_AGENT,
            "discord-tauri/1.0 (+https://discord.com)",
        )
        .send()
        .map_err(|error| {
            eprintln!("failed to fetch favicon: {href} ({error})");
            error
        })
        .ok()?;

    if let Some(content_type) = response.headers().get(CONTENT_TYPE) {
        if content_type
            .to_str()
            .ok()
            .is_some_and(|value| value.contains("image/svg"))
        {
            eprintln!("svg favicon is not supported for window icons: {href}");
            return None;
        }
    }

    response
        .bytes()
        .map(|bytes| bytes.to_vec())
        .map_err(|error| {
            eprintln!("failed to read favicon bytes: {href} ({error})");
            error
        })
        .ok()
}

fn decode_data_url(href: &str) -> Option<Vec<u8>> {
    let (_, payload) = href.split_once(',')?;
    let metadata = &href[..href.len() - payload.len() - 1];

    if metadata.contains("image/svg") {
        eprintln!("svg data-url favicon is not supported for window icons");
        return None;
    }

    if metadata.ends_with(";base64") {
        return STANDARD
            .decode(payload)
            .map_err(|error| {
                eprintln!("failed to decode favicon data-url: {error}");
                error
            })
            .ok();
    }

    None
}

fn main() {
    tauri::Builder::default()
        .manage(FaviconState::default())
        .invoke_handler(tauri::generate_handler![update_favicon])
        .setup(|app| {
            let discord_url = Url::parse(DISCORD_URL).expect("discord login URL must be valid");

            WebviewWindowBuilder::new(app, "main", WebviewUrl::External(discord_url))
                .title(APP_TITLE)
                .decorations(false)
                .zoom_hotkeys_enabled(true)
                .inner_size(1320.0, 860.0)
                .min_inner_size(920.0, 640.0)
                .resizable(true)
                .on_page_load(|window, payload| {
                    if payload.event() == PageLoadEvent::Finished
                        && is_allowed_in_webview(payload.url())
                    {
                        if let Err(error) = window.eval(FAVICON_OBSERVER_SCRIPT) {
                            eprintln!("failed to inject favicon observer: {error}");
                        }
                    }
                })
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

                    if let Err(error) = window.eval(FAVICON_OBSERVER_SCRIPT) {
                        eprintln!("failed to refresh favicon observer: {error}");
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
                .on_new_window(|url, _features| {
                    if is_allowed_in_webview(&url) {
                        NewWindowResponse::Allow
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
