#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod data;
mod logic;
mod models;

use models::*;
use serde_json::json;
use std::sync::Mutex;
use tauri::{Manager, State};

struct AppState(Mutex<Database>);

// ---------------------------------------------------------------------
// Lecturas (una por pantalla / hoja del Excel original)
// ---------------------------------------------------------------------

#[tauri::command]
fn get_dashboard(state: State<AppState>) -> serde_json::Value {
    let db = state.0.lock().unwrap();
    let k = logic::calcular_dashboard(&db);
    json!({
        "total_comisiones_oficina": k.total_comisiones_oficina,
        "matches_disponibles": k.matches_disponibles,
        "expedientes_listos_firma": k.expedientes_listos_firma,
        "pagos_pendientes": k.pagos_pendientes,
    })
}

#[tauri::command]
fn get_asesores(state: State<AppState>) -> serde_json::Value {
    let db = state.0.lock().unwrap();
    let lista = logic::calcular_asesores(&db);
    json!(lista
        .iter()
        .map(|a| json!({
            "id": a.id,
            "nombre": a.nombre,
            "ventas_concretadas": a.ventas_concretadas,
            "alquileres_concretados": a.alquileres_concretados,
            "total_facturado": a.total_facturado,
            "pct_asistencia": a.pct_asistencia,
            "score": a.score,
            "posicion_ranking": a.posicion_ranking,
        }))
        .collect::<Vec<_>>())
}

#[tauri::command]
fn get_matching(state: State<AppState>) -> serde_json::Value {
    let db = state.0.lock().unwrap();
    json!(db
        .matching
        .iter()
        .map(|r| {
            let nivel = logic::calcular_nivel_match(r.precio_lista, r.presupuesto_max);
            json!({
                "id": r.id,
                "cliente_buscador": r.cliente_buscador,
                "asesor_cliente": r.asesor_cliente,
                "tipo_operacion": r.tipo_operacion,
                "zona_deseada": r.zona_deseada,
                "presupuesto_max": r.presupuesto_max,
                "inmueble_matcheado": r.inmueble_matcheado,
                "precio_lista": r.precio_lista,
                "fecha_venc_exclusividad": r.fecha_venc_exclusividad,
                "dias_hasta_venc": logic::dias_hasta(&r.fecha_venc_exclusividad),
                "nivel_match": nivel.etiqueta(),
            })
        })
        .collect::<Vec<_>>())
}

#[tauri::command]
fn get_legal(state: State<AppState>) -> serde_json::Value {
    let db = state.0.lock().unwrap();
    json!(db
        .legal
        .iter()
        .map(|e| json!({
            "cod_inmueble": e.cod_inmueble,
            "propietario": e.propietario,
            "titulo_propiedad": e.titulo_propiedad,
            "cedula_rif": e.cedula_rif,
            "ficha_catastral": e.ficha_catastral,
            "solvencia_municipal": e.solvencia_municipal,
            "liberacion_hipoteca": e.liberacion_hipoteca,
            "borrador_contrato": e.borrador_contrato,
            "estatus_notaria": e.estatus_notaria,
            "estatus_general": logic::calcular_estatus_legal(e),
        }))
        .collect::<Vec<_>>())
}

#[tauri::command]
fn get_embudo(state: State<AppState>) -> serde_json::Value {
    let db = state.0.lock().unwrap();
    json!(db
        .embudo
        .iter()
        .map(|e| json!({
            "id_asesor": e.id_asesor,
            "nombre_asesor": e.nombre_asesor,
            "llamadas_realizadas": e.llamadas_realizadas,
            "citas_captacion": e.citas_captacion,
            "visitas_guiadas": e.visitas_guiadas,
            "ofertas_recibidas": e.ofertas_recibidas,
            "cierres_mes": e.cierres_mes,
            "tasa_conversion": logic::tasa_conversion(e),
            "nivel_actividad": logic::nivel_actividad(e),
        }))
        .collect::<Vec<_>>())
}

#[tauri::command]
fn get_captaciones(state: State<AppState>) -> serde_json::Value {
    let db = state.0.lock().unwrap();
    json!(db
        .captaciones
        .iter()
        .map(|c| json!({
            "cod_inmueble": c.cod_inmueble,
            "tipo_propiedad": c.tipo_propiedad,
            "id_captador": c.id_captador,
            "fecha_captacion": c.fecha_captacion,
            "precio_lista": c.precio_lista,
            "estatus": c.estatus,
            "publicado_web": c.publicado_web,
            "publicado_rrss": c.publicado_rrss,
        }))
        .collect::<Vec<_>>())
}

#[tauri::command]
fn get_cierres(state: State<AppState>) -> serde_json::Value {
    let db = state.0.lock().unwrap();
    let nombre_de = |id: &str| -> String {
        db.asesores
            .iter()
            .find(|a| a.id == id)
            .map(|a| a.nombre.clone())
            .unwrap_or_else(|| id.to_string())
    };
    json!(db
        .cierres
        .iter()
        .map(|c| json!({
            "id": c.id,
            "fecha_cierre": c.fecha_cierre,
            "cod_inmueble": c.cod_inmueble,
            "tipo_operacion": c.tipo_operacion,
            "monto_operacion": c.monto_operacion,
            "id_captador": c.id_captador,
            "nombre_captador": nombre_de(&c.id_captador),
            "id_cerrador": c.id_cerrador,
            "nombre_cerrador": nombre_de(&c.id_cerrador),
            "pct_comision_total": c.pct_comision_total,
            "comision_oficina": logic::comision_oficina(c),
            "pago_asesores": logic::pago_asesores(c),
        }))
        .collect::<Vec<_>>())
}

#[tauri::command]
fn get_finanzas(state: State<AppState>) -> serde_json::Value {
    let db = state.0.lock().unwrap();
    json!({
        "items": db.finanzas.iter().map(|t| json!({
            "id": t.id,
            "fecha": t.fecha,
            "semana": t.semana,
            "tipo_flujo": t.tipo_flujo,
            "categoria": t.categoria,
            "monto": t.monto,
            "estatus_pago": t.estatus_pago,
        })).collect::<Vec<_>>(),
        "total_pagado": logic::total_pagado(&db.finanzas),
        "total_pendiente": logic::total_pendiente(&db.finanzas),
    })
}

#[tauri::command]
fn get_reportes(state: State<AppState>) -> serde_json::Value {
    let db = state.0.lock().unwrap();
    json!(db
        .reportes
        .iter()
        .map(|r| json!({
            "cod_inmueble": r.cod_inmueble,
            "propietario": r.propietario,
            "telefono": r.telefono,
            "id_asesor": r.id_asesor,
            "visitas_agendadas": r.visitas_agendadas,
            "ofertas_recibidas": r.ofertas_recibidas,
            "canales_publicacion": r.canales_publicacion,
            "notas": r.notas,
            "estatus_envio": r.estatus_envio,
        }))
        .collect::<Vec<_>>())
}

/// Lista liviana de asesores (id + nombre) para poblar los <select> de los
/// formularios, en vez de que el usuario tenga que tipear el ID a mano.
#[tauri::command]
fn get_asesores_lista(state: State<AppState>) -> serde_json::Value {
    let db = state.0.lock().unwrap();
    json!(db
        .asesores
        .iter()
        .map(|a| json!({ "id": a.id, "nombre": a.nombre }))
        .collect::<Vec<_>>())
}

// ---------------------------------------------------------------------
// Altas (los 3 formularios que ya existian en la version de terminal)
// ---------------------------------------------------------------------

#[tauri::command]
fn add_cierre(
    state: State<AppState>,
    id: String,
    fecha_cierre: String,
    cod_inmueble: String,
    tipo_operacion: String,
    monto_operacion: f64,
    id_captador: String,
    id_cerrador: String,
    pct_comision_total: f64,
) -> Result<(), String> {
    let mut db = state.0.lock().unwrap();
    db.cierres.push(Cierre {
        id,
        fecha_cierre,
        cod_inmueble,
        tipo_operacion,
        monto_operacion,
        id_captador,
        id_cerrador,
        pct_comision_total,
    });
    data::guardar(&db).map_err(|e| e.to_string())
}

#[tauri::command]
fn add_captacion(
    state: State<AppState>,
    cod_inmueble: String,
    tipo_propiedad: String,
    id_captador: String,
    fecha_captacion: String,
    precio_lista: f64,
    estatus: String,
    publicado_web: bool,
    publicado_rrss: bool,
) -> Result<(), String> {
    let mut db = state.0.lock().unwrap();
    db.captaciones.push(Captacion {
        cod_inmueble,
        tipo_propiedad,
        id_captador,
        fecha_captacion,
        precio_lista,
        estatus,
        publicado_web,
        publicado_rrss,
    });
    data::guardar(&db).map_err(|e| e.to_string())
}

#[tauri::command]
fn add_finanza(
    state: State<AppState>,
    id: String,
    fecha: String,
    semana: String,
    tipo_flujo: String,
    categoria: String,
    monto: f64,
    estatus_pago: String,
) -> Result<(), String> {
    let mut db = state.0.lock().unwrap();
    db.finanzas.push(TransaccionFinanciera {
        id,
        fecha,
        semana,
        tipo_flujo,
        categoria,
        monto,
        estatus_pago,
    });
    data::guardar(&db).map_err(|e| e.to_string())
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

            let db = data::cargar();
            app.manage(AppState(Mutex::new(db)));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_dashboard,
            get_asesores,
            get_matching,
            get_legal,
            get_embudo,
            get_captaciones,
            get_cierres,
            get_finanzas,
            get_reportes,
            get_asesores_lista,
            add_cierre,
            add_captacion,
            add_finanza,
        ])
        .run(tauri::generate_context!())
        .expect("error al correr la app Tauri");
}
