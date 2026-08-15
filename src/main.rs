mod data;
mod logic;
mod models;

use slint::ComponentHandle;
use std::path::PathBuf;

slint::include_modules!();

/// Equivalente a app.path().app_data_dir() de Tauri, pero a mano y sin
/// dependencias extra: una carpeta persistente por SO para guardar los CSV.
fn resolver_data_dir() -> PathBuf {
    #[cfg(target_os = "windows")]
    {
        let base = std::env::var("APPDATA").unwrap_or_else(|_| ".".to_string());
        PathBuf::from(base).join("InmoCore")
    }
    #[cfg(target_os = "macos")]
    {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        PathBuf::from(home).join("Library/Application Support/InmoCore")
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        PathBuf::from(home).join(".local/share/inmocore")
    }
}

fn formatear_dinero(v: f64) -> String {
    format!("${:.2}", v)
}

fn main() {
    let dir = resolver_data_dir();
    std::fs::create_dir_all(&dir).ok();
    data::inicializar_dir(dir.join("data"));

    let db = data::cargar();
    let kpis = logic::calcular_dashboard(&db);

    let ventana = AppWindow::new().expect("no se pudo crear la ventana");

    ventana.set_comisiones_oficina(formatear_dinero(kpis.total_comisiones_oficina).into());
    ventana.set_matches_disponibles(kpis.matches_disponibles.to_string().into());
    ventana.set_expedientes_listos(kpis.expedientes_listos_firma.to_string().into());
    ventana.set_pagos_pendientes(formatear_dinero(kpis.pagos_pendientes).into());

    let ventana_debil = ventana.as_weak();
    ventana.on_actualizar(move || {
        let db = data::cargar();
        let kpis = logic::calcular_dashboard(&db);
        if let Some(v) = ventana_debil.upgrade() {
            v.set_comisiones_oficina(formatear_dinero(kpis.total_comisiones_oficina).into());
            v.set_matches_disponibles(kpis.matches_disponibles.to_string().into());
            v.set_expedientes_listos(kpis.expedientes_listos_firma.to_string().into());
            v.set_pagos_pendientes(formatear_dinero(kpis.pagos_pendientes).into());
        }
    });

    ventana.run().expect("error al correr la app Slint");
}
