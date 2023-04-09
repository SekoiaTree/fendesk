#![cfg_attr(
all(not(debug_assertions), target_os = "windows"),
windows_subsystem = "windows"
)]

use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::sync::{Mutex, MutexGuard};
use std::time::Instant;
use dialog::{DialogBox, FileSelectionMode};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tauri::{AppHandle, ClipboardManager, Manager, Window, WindowEvent, Wry};

struct TimeoutInterrupt {
    start: Instant,
    timeout: u128,
}

impl TimeoutInterrupt {
    fn new_with_timeout(timeout: u128) -> Self {
        Self {
            start: Instant::now(),
            timeout,
        }
    }
}

impl fend_core::Interrupt for TimeoutInterrupt {
    fn should_interrupt(&self) -> bool {
        Instant::now().duration_since(self.start).as_millis() > self.timeout
    }
}

struct FendContext(Mutex<fend_core::Context>);

struct SettingsState(Mutex<serde_json::Value>);

#[tauri::command]
fn copy_to_clipboard(value: String, app_handle: AppHandle<Wry>) {
    match app_handle.clipboard_manager().write_text(value) {
        Ok(_) => {}
        Err(x) => {
            eprintln!("Failed to copy to clipboard: {}", x);
        }
    }
}

#[tauri::command]
fn load_from_file() -> Result<(Vec<String>, bool), String> {
    let dialog = dialog::FileSelection::new("Select file to open")
        .mode(FileSelectionMode::Open)
        .title("Open a list of things to run")
        .show().map_err(|x| x.to_string())?;

    if let Some(x) = dialog {
        let file = BufReader::new(File::open(x).map_err(|e| e.to_string())?);
        return Ok((file.lines().filter_map(|x| x.ok()).collect(), false));
    }

    Ok((vec![], true))
}

#[tauri::command]
fn save_to_file(input: Vec<String>) -> Result<bool, String> {
    let dialog = dialog::FileSelection::new("Select file to save to")
        .mode(FileSelectionMode::Save)
        .title("Saving input")
        .show().map_err(|x| x.to_string())?;

    if let Some(x) = dialog {
        let mut file = File::create(x).map_err(|e| e.to_string())?;

        for i in input {
            writeln!(file, "{}", i).map_err(|e| e.to_string())?;
        }

        Ok(true)
    } else {
        Ok(false)
    }
}

#[tauri::command]
fn fend_completion(value: String) -> Option<String> {
    let (_, completions) = fend_core::get_completions_for_prefix(value.as_str());

    Some(completions.first()?.insert().to_string())
}

#[tauri::command]
fn fend_prompt(value: String, timeout: i64, state: tauri::State<FendContext>) -> Result<String, String> {
    let mut context = (*state).0.lock().unwrap();
    fend_prompt_internal(value, timeout, &mut (*context))
}


#[tauri::command]
fn fend_preview_prompt(value: String, timeout: i64, state: tauri::State<FendContext>) -> Result<String, String> {
    let mut context = (*state).0.lock().unwrap().clone();
    fend_prompt_internal(value, timeout, &mut context)
}

fn fend_prompt_internal(value: String, timeout: i64, context: &mut fend_core::Context) -> Result<String, String> {
    let interrupt = TimeoutInterrupt::new_with_timeout(timeout as u128);
    fend_core::evaluate_with_interrupt(&*value, context, &interrupt)
        .map(|res| {
            if res.is_unit_type() {
                "".to_string()
            } else {
                res.get_main_result().to_string()
            }
        })
}

#[tauri::command]
fn quit(window: Window<Wry>) {
    window.close().expect("Failed to close successfully. Yeah, I don't know either.")
}

#[tauri::command]
async fn open_settings(handle: AppHandle<Wry>) {
    let settings_window = tauri::WindowBuilder::new(
        &handle,
        "external", /* the unique window label */
        tauri::WindowUrl::App("settings.html".parse().unwrap()),
    ).title("fendesk settings").build().unwrap();

    settings_window.on_window_event(move |x| {
        if matches!(x, WindowEvent::CloseRequested { .. }) {
            handle.emit_all("settings-closed", ()).unwrap();
        }
    })
}

const SETTINGS_CORRUPTED: &str = "The settings were corrupted. Should never happen, please report.";

#[tauri::command]
fn set_setting(id: String, value: serde_json::Value, settings: tauri::State<SettingsState>) {
    settings.0.lock().expect(SETTINGS_CORRUPTED)[id] = value;
}

#[tauri::command]
fn save_settings(settings: tauri::State<SettingsState>) -> Result<(), String> {
    if let Some(x) = tauri::api::path::config_dir() {
        let settings : MutexGuard<serde_json::Value> = settings.0.lock().expect(SETTINGS_CORRUPTED);
        fs::create_dir_all(x.join("fendesk")).map_err(|e| e.to_string())?;
        let mut file = File::create(x.join("fendesk/settings.json")).map_err(|e| e.to_string())?;
        file.write(serde_json::to_string(&*settings).map_err(|e| e.to_string())?.as_bytes()).map_err(|e| e.to_string())?;
        Ok(())
    } else {
        Err("Configuration folder not found!".to_string())
    }
}

#[tauri::command]
fn get_settings(settings: tauri::State<SettingsState>) -> serde_json::Value {
    settings.0.lock().expect(SETTINGS_CORRUPTED).clone()
}

fn main() {
    let context = create_context();
    let default_settings = json!({
        "ctrl_d_closes": true,
        "ctrl_w_closes": false,
        "save_back_count": -1,
        "ctrl_c_behavior": "input",
        "global_inputs": "",
    });

    tauri::Builder::default()
        .setup(|app| {
            let main = app.get_window("main").unwrap();
            let handle = app.handle();
            main.on_window_event(move |x| {
                if matches!(x, WindowEvent::CloseRequested { .. }) {
                    handle.exit(0);
                }
            });

            Ok(())
        })
        .manage(FendContext(Mutex::new(context)))
        .manage(SettingsState(Mutex::new(get_saved_settings().unwrap_or(default_settings))))
        .invoke_handler(tauri::generate_handler![
            setup_exchanges, fend_prompt, fend_preview_prompt, fend_completion, // core fend
            quit, copy_to_clipboard, save_to_file, load_from_file, // ctrl- shortcuts
            open_settings, set_setting, get_settings, save_settings // settings
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn get_saved_settings() -> Result<serde_json::Value, ()> {
    if let Some(x) = tauri::api::path::config_dir() {
        fs::create_dir_all(x.join("fendesk")).map_err(|_| ())?;
        let file = File::open(x.join("fendesk/settings.json")).map_err(|_| ())?;
        serde_json::from_reader(file).map_err(|_| ())
    } else {
        Err(())
    }
}

fn create_context() -> fend_core::Context {
    let mut context = fend_core::Context::new();
    let current_time = chrono::Local::now();
    context.set_current_time_v1(current_time.timestamp_millis() as u64, current_time.offset().local_minus_utc() as i64);

    context.set_random_u32_fn(rand::random);

    // TODO: this is currently disabled because with large ranges it completely covers the screen, but otherwise I like it more than the default. Fix that in the fork.
    //context.set_output_mode_terminal();

    return context;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ExchangeRates {
    date: String,
    base: String,
    rates: HashMap<String, f64>,
}

#[tauri::command]
async fn setup_exchanges(state: tauri::State<'_, FendContext>) -> Result<(), ()> {
    let exchanges = get_exchanges().await;
    if let Some(x) = exchanges {
        (*state).0.lock().unwrap().set_exchange_rate_handler_v1(move |currency: &str| {
            match x.rates.get(&currency.to_string()) {
                None => Err(Box::<dyn std::error::Error + Send + Sync>::from(format!("Failed to get exchange rate for {}", currency))),
                Some(v) => Ok(*v)
            }
        })
    } else {
        eprintln!("Failed to get exchange rates!");
    }

    Ok(())
}

async fn get_exchanges() -> Option<ExchangeRates> {
    if let Some(x) = tauri::api::path::cache_dir() {
        if let Some(x) = read_cached_exchanges(x) {
            return Some(x);
        }
    }
    get_exchanges_online().await
}

fn read_cached_exchanges(cache_dir: PathBuf) -> Option<ExchangeRates> {
    let x = File::open(cache_dir.join("fendesk/exchanges.txt")).ok()?;
    let buffered = BufReader::new(x);
    let read: ExchangeRates = serde_json::from_reader(buffered).ok()?;
    if read.date == format!("{}", chrono::Local::now().format("%Y-%m-%d")) {
        Some(read)
    } else {
        None
    }
}

/// Get the exchanges from online. Should generally only be called once per day.
async fn get_exchanges_online() -> Option<ExchangeRates> {
    let response = reqwest::get("https://api.vatcomply.com/rates?base=USD").await.ok()?.bytes().await.ok()?;
    let json = serde_json::from_slice(response.as_ref());
    json.ok()
}