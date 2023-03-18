#![cfg_attr(
all(not(debug_assertions), target_os = "windows"),
windows_subsystem = "windows"
)]

use std::sync::Mutex;
use std::time::Instant;
use arboard::Clipboard;
use dialog::{DialogBox, FileSelectionMode};
use tauri::{Manager, Menu, WindowEvent, Wry};

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
struct SettingsState(bool);

#[tauri::command]
fn copy_to_clipboard(value: String) {
    let mut clipboard = Clipboard::new().unwrap();
    clipboard.set_text(value).unwrap();
}

#[tauri::command]
fn save_to_file(_state: tauri::State<FendContext>) -> Result<(), String> {
    let dialog = dialog::FileSelection::new("Select file to save to")
        .mode(FileSelectionMode::Save)
        .title("Saving variables")
        .show().map_err(|x| x.to_string())?;

    // TODO: error reporting; toasts for error, cancelled, and success
    if let Some(_x) = dialog {
        todo!() // we need to fork fend to get access to variables and such.
    }

    Ok(())
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
fn quit(window: tauri::Window<Wry>) {
    window.close().expect("Failed to close successfully. Yeah, I don't know either.")
}

#[tauri::command]
async fn open_settings(handle: tauri::AppHandle<Wry>) {
    let settings_window = tauri::WindowBuilder::new(
        &handle,
        "external", /* the unique window label */
        tauri::WindowUrl::App("settings.html".parse().unwrap())
    ).build().unwrap();

    settings_window.on_window_event(move |x| {
        if matches!(x, WindowEvent::CloseRequested { .. }) {
            handle.emit_all("settings-closed", ()).unwrap();
        }
    })
}

fn main() {
    let context = create_context();

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
        .manage(SettingsState(false))
        .invoke_handler(tauri::generate_handler![
            fend_prompt, fend_preview_prompt, // core fend
            quit, copy_to_clipboard, save_to_file, // ctrl- shortcuts
            open_settings // settings
        ])
        .menu(Menu::os_default("fendesk"))
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn create_context() -> fend_core::Context {
    // TODO: read from config
    let mut context = fend_core::Context::new();
    let current_time = chrono::Local::now();
    context.set_current_time_v1(current_time.timestamp_millis() as u64, current_time.offset().local_minus_utc() as i64);

    context.set_random_u32_fn(rand::random);

    // TODO: this is currently disabled because with large ranges it completely covers the screen, but otherwise I like it more than the default. Fix that in the fork.
    //context.set_output_mode_terminal();

    return context;
}