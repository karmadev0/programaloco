mod data;
mod logic;
mod models;

use models::{Captacion, Cierre, TransaccionFinanciera};
use slint::{ModelRc, SharedString, VecModel};
use std::path::PathBuf;

slint::include_modules!();

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

fn dinero(v: f64) -> String {
    format!("${:.2}", v)
}

fn porcentaje(v: f64) -> String {
    format!("{:.0}%", v * 100.0)
}

fn bool_str(b: bool) -> String {
    if b { "SI".into() } else { "NO".into() }
}

fn parse_f64(s: &str) -> f64 {
    s.trim().replace(',', ".").parse::<f64>().unwrap_or(0.0)
}

fn fila(celdas: Vec<String>) -> Fila {
    let items: Vec<SharedString> = celdas.into_iter().map(SharedString::from).collect();
    Fila { celdas: ModelRc::new(VecModel::from(items)) }
}

fn filas_modelo(filas: Vec<Fila>) -> ModelRc<Fila> {
    ModelRc::new(VecModel::from(filas))
}

// ---------------------------------------------------------------------
// Carga por pantalla (equivalente a cada get_* de main.rs en la version
// Tauri, pero escribiendo directo a las propiedades de la ventana Slint
// en vez de devolver JSON).
// ---------------------------------------------------------------------

fn cargar_dashboard(v: &AppWindow) {
    let db = data::cargar();
    let k = logic::calcular_dashboard(&db);
    v.set_comisiones_oficina(dinero(k.total_comisiones_oficina).into());
    v.set_matches_disponibles(k.matches_disponibles.to_string().into());
    v.set_expedientes_listos(k.expedientes_listos_firma.to_string().into());
    v.set_pagos_pendientes(dinero(k.pagos_pendientes).into());
}

fn cargar_asesores(v: &AppWindow) {
    let db = data::cargar();
    let lista = logic::calcular_asesores(&db);
    let filas: Vec<Fila> = lista
        .iter()
        .map(|a| {
            fila(vec![
                a.id.clone(),
                a.nombre.clone(),
                a.ventas_concretadas.to_string(),
                a.alquileres_concretados.to_string(),
                dinero(a.total_facturado),
                porcentaje(a.pct_asistencia),
                format!("{:.1}", a.score),
                a.posicion_ranking.to_string(),
            ])
        })
        .collect();
    v.set_asesores_rows(filas_modelo(filas));
}

fn cargar_matching(v: &AppWindow) {
    let db = data::cargar();
    let filas: Vec<Fila> = db
        .matching
        .iter()
        .map(|r| {
            let nivel = logic::calcular_nivel_match(r.precio_lista, r.presupuesto_max);
            fila(vec![
                r.id.clone(),
                r.cliente_buscador.clone(),
                r.asesor_cliente.clone(),
                r.tipo_operacion.clone(),
                r.zona_deseada.clone(),
                dinero(r.presupuesto_max),
                r.inmueble_matcheado.clone(),
                dinero(r.precio_lista),
                r.fecha_venc_exclusividad.clone(),
                logic::dias_hasta(&r.fecha_venc_exclusividad).to_string(),
                nivel.etiqueta().to_string(),
            ])
        })
        .collect();
    v.set_matching_rows(filas_modelo(filas));
}

fn cargar_legal(v: &AppWindow) {
    let db = data::cargar();
    let filas: Vec<Fila> = db
        .legal
        .iter()
        .map(|e| {
            fila(vec![
                e.cod_inmueble.clone(),
                e.propietario.clone(),
                bool_str(e.titulo_propiedad),
                bool_str(e.cedula_rif),
                bool_str(e.ficha_catastral),
                bool_str(e.solvencia_municipal),
                e.liberacion_hipoteca.clone(),
                e.borrador_contrato.clone(),
                e.estatus_notaria.clone(),
                logic::calcular_estatus_legal(e).to_string(),
            ])
        })
        .collect();
    v.set_legal_rows(filas_modelo(filas));
}

fn cargar_embudo(v: &AppWindow) {
    let db = data::cargar();
    let filas: Vec<Fila> = db
        .embudo
        .iter()
        .map(|e| {
            fila(vec![
                e.id_asesor.clone(),
                e.nombre_asesor.clone(),
                e.llamadas_realizadas.to_string(),
                e.citas_captacion.to_string(),
                e.visitas_guiadas.to_string(),
                e.ofertas_recibidas.to_string(),
                e.cierres_mes.to_string(),
                porcentaje(logic::tasa_conversion(e)),
                logic::nivel_actividad(e).to_string(),
            ])
        })
        .collect();
    v.set_embudo_rows(filas_modelo(filas));
}

fn cargar_cierres(v: &AppWindow) {
    let db = data::cargar();
    let nombre_de = |id: &str| -> String {
        db.asesores
            .iter()
            .find(|a| a.id == id)
            .map(|a| a.nombre.clone())
            .unwrap_or_else(|| id.to_string())
    };
    let filas: Vec<Fila> = db
        .cierres
        .iter()
        .map(|c| {
            fila(vec![
                c.id.clone(),
                c.fecha_cierre.clone(),
                c.cod_inmueble.clone(),
                c.tipo_operacion.clone(),
                dinero(c.monto_operacion),
                nombre_de(&c.id_captador),
                nombre_de(&c.id_cerrador),
                porcentaje(c.pct_comision_total),
                dinero(logic::comision_oficina(c)),
                dinero(logic::pago_asesores(c)),
            ])
        })
        .collect();
    v.set_cierres_rows(filas_modelo(filas));
}

fn cargar_finanzas(v: &AppWindow) {
    let db = data::cargar();
    let filas: Vec<Fila> = db
        .finanzas
        .iter()
        .map(|t| {
            fila(vec![
                t.id.clone(),
                t.fecha.clone(),
                t.semana.clone(),
                t.tipo_flujo.clone(),
                t.categoria.clone(),
                dinero(t.monto),
                t.estatus_pago.clone(),
            ])
        })
        .collect();
    v.set_finanzas_rows(filas_modelo(filas));
    v.set_finanzas_total_pagado(dinero(logic::total_pagado(&db.finanzas)).into());
    v.set_finanzas_total_pendiente(dinero(logic::total_pendiente(&db.finanzas)).into());
}

fn cargar_captaciones(v: &AppWindow) {
    let db = data::cargar();
    let filas: Vec<Fila> = db
        .captaciones
        .iter()
        .map(|c| {
            fila(vec![
                c.cod_inmueble.clone(),
                c.tipo_propiedad.clone(),
                c.id_captador.clone(),
                c.fecha_captacion.clone(),
                dinero(c.precio_lista),
                c.estatus.clone(),
                bool_str(c.publicado_web),
                bool_str(c.publicado_rrss),
            ])
        })
        .collect();
    v.set_captaciones_rows(filas_modelo(filas));
}

fn cargar_reportes(v: &AppWindow) {
    let db = data::cargar();
    let filas: Vec<Fila> = db
        .reportes
        .iter()
        .map(|r| {
            fila(vec![
                r.cod_inmueble.clone(),
                r.propietario.clone(),
                r.telefono.clone(),
                r.id_asesor.clone(),
                r.visitas_agendadas.to_string(),
                r.ofertas_recibidas.to_string(),
                r.canales_publicacion.clone(),
                r.notas.clone(),
                r.estatus_envio.clone(),
            ])
        })
        .collect();
    v.set_reportes_rows(filas_modelo(filas));
}

fn main() {
    let dir = resolver_data_dir();
    std::fs::create_dir_all(&dir).ok();
    data::inicializar_dir(dir.join("data"));

    let v = AppWindow::new().expect("no se pudo crear la ventana");

    cargar_dashboard(&v);
    cargar_asesores(&v);
    cargar_matching(&v);
    cargar_legal(&v);
    cargar_embudo(&v);
    cargar_cierres(&v);
    cargar_finanzas(&v);
    cargar_captaciones(&v);
    cargar_reportes(&v);

    macro_rules! conectar {
        ($cb:ident, $fun:ident) => {
            let debil = v.as_weak();
            v.$cb(move || {
                if let Some(v) = debil.upgrade() {
                    $fun(&v);
                }
            });
        };
    }
    conectar!(on_actualizar_dashboard, cargar_dashboard);
    conectar!(on_actualizar_asesores, cargar_asesores);
    conectar!(on_actualizar_matching, cargar_matching);
    conectar!(on_actualizar_legal, cargar_legal);
    conectar!(on_actualizar_embudo, cargar_embudo);
    conectar!(on_actualizar_cierres, cargar_cierres);
    conectar!(on_actualizar_finanzas, cargar_finanzas);
    conectar!(on_actualizar_captaciones, cargar_captaciones);
    conectar!(on_actualizar_reportes, cargar_reportes);

    // --- Alta: Cierre ---
    let debil = v.as_weak();
    v.on_guardar_cierre(move || {
        let v = match debil.upgrade() {
            Some(v) => v,
            None => return,
        };
        let id = v.get_cierre_id().to_string();
        if id.trim().is_empty() {
            v.set_mensaje_cierre("Falta el ID".into());
            return;
        }
        let mut db = data::cargar();
        db.cierres.push(Cierre {
            id,
            fecha_cierre: v.get_cierre_fecha().to_string(),
            cod_inmueble: v.get_cierre_cod_inmueble().to_string(),
            tipo_operacion: v.get_cierre_tipo_operacion().to_string(),
            monto_operacion: parse_f64(&v.get_cierre_monto()),
            id_captador: v.get_cierre_id_captador().to_string(),
            id_cerrador: v.get_cierre_id_cerrador().to_string(),
            pct_comision_total: parse_f64(&v.get_cierre_pct_comision()),
        });
        if let Err(e) = data::guardar(&db) {
            v.set_mensaje_cierre(format!("Error al guardar: {e}").into());
            return;
        }
        v.set_cierre_id("".into());
        v.set_cierre_fecha("".into());
        v.set_cierre_cod_inmueble("".into());
        v.set_cierre_tipo_operacion("".into());
        v.set_cierre_monto("".into());
        v.set_cierre_id_captador("".into());
        v.set_cierre_id_cerrador("".into());
        v.set_cierre_pct_comision("".into());
        v.set_mensaje_cierre("Guardado ✓".into());
        cargar_cierres(&v);
        cargar_dashboard(&v);
        cargar_asesores(&v);
    });

    // --- Alta: Captación ---
    let debil = v.as_weak();
    v.on_guardar_captacion(move || {
        let v = match debil.upgrade() {
            Some(v) => v,
            None => return,
        };
        let cod = v.get_captacion_cod_inmueble().to_string();
        if cod.trim().is_empty() {
            v.set_mensaje_captacion("Falta el código de inmueble".into());
            return;
        }
        let mut db = data::cargar();
        db.captaciones.push(Captacion {
            cod_inmueble: cod,
            tipo_propiedad: v.get_captacion_tipo_propiedad().to_string(),
            id_captador: v.get_captacion_id_captador().to_string(),
            fecha_captacion: v.get_captacion_fecha().to_string(),
            precio_lista: parse_f64(&v.get_captacion_precio()),
            estatus: v.get_captacion_estatus().to_string(),
            publicado_web: v.get_captacion_web(),
            publicado_rrss: v.get_captacion_rrss(),
        });
        if let Err(e) = data::guardar(&db) {
            v.set_mensaje_captacion(format!("Error al guardar: {e}").into());
            return;
        }
        v.set_captacion_cod_inmueble("".into());
        v.set_captacion_tipo_propiedad("".into());
        v.set_captacion_id_captador("".into());
        v.set_captacion_fecha("".into());
        v.set_captacion_precio("".into());
        v.set_captacion_estatus("".into());
        v.set_captacion_web(false);
        v.set_captacion_rrss(false);
        v.set_mensaje_captacion("Guardado ✓".into());
        cargar_captaciones(&v);
    });

    // --- Alta: Finanza ---
    let debil = v.as_weak();
    v.on_guardar_finanza(move || {
        let v = match debil.upgrade() {
            Some(v) => v,
            None => return,
        };
        let id = v.get_finanza_id().to_string();
        if id.trim().is_empty() {
            v.set_mensaje_finanza("Falta el ID".into());
            return;
        }
        let mut db = data::cargar();
        db.finanzas.push(TransaccionFinanciera {
            id,
            fecha: v.get_finanza_fecha().to_string(),
            semana: v.get_finanza_semana().to_string(),
            tipo_flujo: v.get_finanza_tipo_flujo().to_string(),
            categoria: v.get_finanza_categoria().to_string(),
            monto: parse_f64(&v.get_finanza_monto()),
            estatus_pago: v.get_finanza_estatus().to_string(),
        });
        if let Err(e) = data::guardar(&db) {
            v.set_mensaje_finanza(format!("Error al guardar: {e}").into());
            return;
        }
        v.set_finanza_id("".into());
        v.set_finanza_fecha("".into());
        v.set_finanza_semana("".into());
        v.set_finanza_tipo_flujo("".into());
        v.set_finanza_categoria("".into());
        v.set_finanza_monto("".into());
        v.set_finanza_estatus("".into());
        v.set_mensaje_finanza("Guardado ✓".into());
        cargar_finanzas(&v);
        cargar_dashboard(&v);
    });

    v.run().expect("error al correr la app Slint");
}
