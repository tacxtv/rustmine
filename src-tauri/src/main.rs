// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::Manager;

fn main() {
    tauri::Builder::default()
        .setup(|app|{
            let main_window = app.get_window("main").unwrap();
            main_window.set_skip_taskbar(true)?;
        //     let splashscreen_window = tauri::WindowBuilder::new(
        //         app,
        //         "splashscreen",
        //         tauri::WindowUrl::default(),
        //     ).build()?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
          my_custom_command,
        ])
        // .plugin()
        .run(tauri::generate_context!())
        .expect("Error while running RustMine application");
}

#[tauri::command]
fn my_custom_command(version: String) {
    println!("J'ai été invoqué à partir de JS ! {}", version);
    // File::create("test.txt").unwrap();
}

// #[tauri::command]
// async fn window_create(handle: AppHandle) {
//     println!("Minimizing window");
//     let splashscreen_window = tauri::WindowBuilder::new(
//         &handle,
//         "splashscreen",
//         tauri::WindowUrl::default(),
//     ).build();
//     // tauri::Window::get_window(&(), "").unwrap().minimize().expect("TODO: panic message");
// }
