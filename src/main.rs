#![cfg_attr(
all(not(debug_assertions), target_os = "windows"),
windows_subsystem = "windows"
)]

use std::sync::Mutex;
use std::time::Instant;

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

#[tauri::command]
fn fend_prompt(value: String, timeout: i64, state: tauri::State<FendContext>) -> Result<String, String> {
    let mut context = (*state).0.lock().unwrap();
    fend_prompt_internal(value, timeout, &mut (*context))
}


#[tauri::command]
fn fend_preview_prompt(value: String, timeout: i64, state: tauri::State<FendContext>) -> Result<String, String> {
    let mut context = (*state).0.lock().unwrap().clone();
    context.disable_rng();
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

fn main() {
    tauri::Builder::default()
        .manage(FendContext(Mutex::new(fend_core::Context::new())))
        .invoke_handler(tauri::generate_handler![fend_prompt, fend_preview_prompt])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
