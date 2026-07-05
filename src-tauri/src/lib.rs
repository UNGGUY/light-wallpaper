// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
//
//
//

#[cfg_attr(mobile, tauri::mobile_entry_point)]
#[tauri::command]
fn hello_world() {
    println!("I was invoked from JavaScript!");
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![hello_world])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
