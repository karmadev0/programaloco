#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod data;
mod logic;
mod models;
mod modulos;
mod seed;

use models::Database;
use serde_json::json;
use std::sync::Mutex;
use tauri::{Manager, State};

struct AppState(Mutex<Database>);

// ---------------------------------------------------------------------
// Lecturas. Toda la logica de KAVELA vive en modulos.rs / logic.rs (la
// misma que usa la version Slint); aqui solo se convierte a JSON.
// ---------------------------------------------------------------------

fn kpis_json(v: Vec<(String, String)>) -> serde_json::Value {
    json!(v.into_iter().map(|(e, val)| json!({ "etiqueta": e, "valor": val })).collect::<Vec<_>>())
}

#[tauri::command]
fn get_dashboard(state: State<AppState>) -> serde_json::Value {
    let db = state.0.lock().unwrap();
    let d = modulos::dashboard(&db);
    json!({
        "kpis": kpis_json(d.kpis),
        "honor": kpis_json(d.honor),
        "headers": d.headers,
        "filas": d.filas,
    })
}

#[tauri::command]
fn get_vista(state: State<AppState>, modulo: i32) -> serde_json::Value {
    let db = state.0.lock().unwrap();
    let v = modulos::vista(&db, modulo);
    json!({
        "titulo": v.titulo,
        "ayuda": v.ayuda,
        "headers": v.headers,
        "filas": v.filas,
        "puede_crear": v.puede_crear,
        "puede_editar": v.puede_editar,
        "puede_eliminar": v.puede_eliminar,
    })
}

#[tauri::command]
fn get_campos(state: State<AppState>, modulo: i32, idx: Option<usize>) -> Result<serde_json::Value, String> {
    let db = state.0.lock().unwrap();
    let c = modulos::campos(&db, modulo, idx)?;
    Ok(json!(c
        .into_iter()
        .map(|c| json!({ "etiqueta": c.etiqueta, "valor": c.valor, "opciones": c.opciones }))
        .collect::<Vec<_>>()))
}

#[tauri::command]
fn get_graficos(state: State<AppState>) -> serde_json::Value {
    let db = state.0.lock().unwrap();
    json!(modulos::graficos(&db)
        .into_iter()
        .map(|g| {
            let max = g.barras.iter().map(|b| b.1).fold(0.0_f64, f64::max);
            json!({
                "titulo": g.titulo,
                "barras": g.barras.into_iter().map(|(e, v, t)| json!({
                    "etiqueta": e,
                    "proporcion": if max > 0.0 { v / max } else { 0.0 },
                    "texto": t,
                })).collect::<Vec<_>>(),
            })
        })
        .collect::<Vec<_>>())
}

#[tauri::command]
fn consultar_guardia(state: State<AppState>, fecha: String) -> String {
    let db = state.0.lock().unwrap();
    modulos::consultar_guardia(&db, &fecha)
}

// ---------------------------------------------------------------------
// Escrituras (alta / edicion / baja de cualquier modulo)
// ---------------------------------------------------------------------

#[tauri::command]
fn guardar_registro(
    state: State<AppState>,
    modulo: i32,
    idx: Option<usize>,
    valores: Vec<String>,
) -> Result<String, String> {
    let mut db = state.0.lock().unwrap();
    let msg = modulos::guardar(&mut db, modulo, idx, &valores)?;
    data::guardar(&db).map_err(|e| format!("No se pudo guardar en disco: {}", e))?;
    Ok(msg)
}

#[tauri::command]
fn eliminar_registro(state: State<AppState>, modulo: i32, idx: usize) -> Result<(), String> {
    let mut db = state.0.lock().unwrap();
    modulos::eliminar(&mut db, modulo, idx)?;
    data::guardar(&db).map_err(|e| format!("No se pudo guardar en disco: {}", e))
}

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            // Carpeta de datos de usuario segun el SO (persistente aunque se
            // reinstale la app en Program Files / Applications / etc).
            let dir = app
                .path()
                .app_data_dir()
                .expect("no se pudo resolver el directorio de datos de usuario");
            std::fs::create_dir_all(&dir).ok();
            data::inicializar_dir(dir.join("data"));

            // La primera vez carga los datos iniciales del Excel de Kavela.
            let db = data::cargar();
            app.manage(AppState(Mutex::new(db)));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_dashboard,
            get_vista,
            get_campos,
            get_graficos,
            consultar_guardia,
            guardar_registro,
            eliminar_registro,
        ])
        .run(tauri::generate_context!())
        .expect("error al correr la app Tauri");
}
