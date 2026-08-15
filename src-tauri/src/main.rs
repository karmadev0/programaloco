#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

// Paso 1 del roadmap: scaffold minimo, sin logica de negocio todavia.
// models.rs / logic.rs / data.rs se migran en el paso 2.

#[tauri::command]
fn ping() -> String {
    "InmoCore backend vivo".into()
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![ping])
        .run(tauri::generate_context!())
        .expect("error al correr la app Tauri");
}
