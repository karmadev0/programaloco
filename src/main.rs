mod data;
mod logic;
mod models;

use models::{
    Asesor, Captacion, Cierre, EmbudoAsesor, ExpedienteLegal, ReportePropietario, Requerimiento,
    TransaccionFinanciera,
};
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
    // Normaliza -0.0 -> 0.0 para que nunca se muestre "-0.00" (el signo de
    // cero negativo de punto flotante no tiene significado para el usuario).
    let v = if v == 0.0 { 0.0 } else { v };
    format!("${:.2}", v)
}

fn porcentaje(v: f64) -> String {
    format!("{:.0}%", v * 100.0)
}

fn bool_str(b: bool) -> String {
    if b { "SI".into() } else { "NO".into() }
}

/// Intenta parsear un numero (acepta coma o punto decimal). Devuelve None
/// si el campo esta vacio o no es un numero valido, para que el llamador
/// pueda bloquear el guardado con un mensaje claro en vez de guardar un 0
/// silencioso.
fn parse_numero(s: &str) -> Option<f64> {
    let t = s.trim().replace(',', ".");
    if t.is_empty() {
        return None;
    }
    t.parse::<f64>().ok()
}

/// Arma "YYYY-MM-DD" a partir de los 3 selectores de fecha (dia/mes/anio).
fn fecha_str(anio: i32, mes: i32, dia: i32) -> String {
    format!("{:04}-{:02}-{:02}", anio, mes, dia)
}

/// Descompone "YYYY-MM-DD" en (anio, mes, dia) para precargar los 3
/// selectores al editar un registro existente. Si el formato es invalido
/// (o esta vacio), cae en la fecha de hoy.
fn partes_de_fecha(s: &str) -> (i32, i32, i32) {
    let partes: Vec<&str> = s.trim().split('-').collect();
    if partes.len() == 3 {
        if let (Ok(y), Ok(m), Ok(d)) = (
            partes[0].parse::<i32>(),
            partes[1].parse::<i32>(),
            partes[2].parse::<i32>(),
        ) {
            return (y, m, d);
        }
    }
    hoy_dma()
}

/// (anio, mes, dia) de hoy, para precargar formularios "Nuevo".
fn hoy_dma() -> (i32, i32, i32) {
    let (y, m, d) = logic::hoy_civil();
    (y as i32, m as i32, d as i32)
}

/// Exige que un campo numerico este presente y sea valido antes de guardar.
/// Si falla, deja el mensaje de error en la pantalla correspondiente y
/// hace `return` de la clausura donde se invoca (bloquea el guardado).
macro_rules! numero_obligatorio {
    ($v:expr, $texto:expr, $campo:literal, $set_msg:ident) => {
        match parse_numero(&$texto) {
            Some(n) => n,
            None => {
                $v.$set_msg(format!("El campo '{}' debe ser un número válido.", $campo).into());
                return;
            }
        }
    };
}

fn existe_asesor(db: &models::Database, id: &str) -> bool {
    db.asesores.iter().any(|a| a.id == id)
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

    // Lista simple id+nombre para el autocompletado de CampoAsesor (ID
    // Captador / ID Cerrador / ID Asesor, etc). Se refresca cada vez que se
    // recarga BD Asesores, asi que siempre queda al dia.
    let opciones: Vec<AsesorOpcion> = db
        .asesores
        .iter()
        .map(|a| AsesorOpcion {
            id: a.id.clone().into(),
            nombre: a.nombre.clone().into(),
        })
        .collect();
    v.set_asesores_opciones(ModelRc::new(VecModel::from(opciones)));
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

    let disponibles = logic::contar_por_estatus(&db.captaciones, "Disponible");
    let cerradas = logic::contar_por_estatus(&db.captaciones, "Cerrado");
    let otros = db.captaciones.len() as i64 - disponibles - cerradas;
    v.set_captaciones_disponibles(disponibles.to_string().into());
    v.set_captaciones_cerradas(cerradas.to_string().into());
    v.set_captaciones_otros(otros.to_string().into());
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

    // Autocompletado: dado lo que se va tipeando en un campo "ID Captador"
    // / "ID Cerrador" / etc, devuelve los asesores cuyo nombre o ID
    // contienen ese texto (sin importar mayus/minus). El filtrado se hace
    // aca porque el lenguaje .slint no tiene una funcion "contains" para
    // strings.
    v.on_buscar_asesores(move |texto| {
        let db = data::cargar();
        let t = texto.trim().to_lowercase();
        if t.is_empty() {
            return ModelRc::new(VecModel::from(Vec::<AsesorOpcion>::new()));
        }
        let mut coincidencias: Vec<AsesorOpcion> = db
            .asesores
            .iter()
            .filter(|a| {
                a.nombre.to_lowercase().contains(&t) || a.id.to_lowercase().contains(&t)
            })
            .map(|a| AsesorOpcion {
                id: a.id.clone().into(),
                nombre: a.nombre.clone().into(),
            })
            .collect();
        coincidencias.sort_by(|a, b| a.nombre.cmp(&b.nombre));
        coincidencias.truncate(8);
        ModelRc::new(VecModel::from(coincidencias))
    });

    // --- "Nuevo X": limpia el form y el indice de edicion antes de abrir,
    // asi nunca se pisa un registro existente por error.
    let debil = v.as_weak();
    v.on_nuevo_asesor(move || {
        if let Some(v) = debil.upgrade() {
            let (y, m, d) = hoy_dma();
            v.set_asesor_id("".into());
            v.set_asesor_nombre("".into());
            v.set_asesor_fecha_ingreso_anio(y);
            v.set_asesor_fecha_ingreso_mes(m);
            v.set_asesor_fecha_ingreso_dia(d);
            v.set_asesor_talleres("".into());
            v.set_asesor_editando(-1);
            v.set_mensaje_asesor("".into());
            v.set_mostrar_form_asesor(true);
        }
    });
    let debil = v.as_weak();
    v.on_nuevo_matching(move || {
        if let Some(v) = debil.upgrade() {
            let (y, m, d) = hoy_dma();
            v.set_matching_id("".into());
            v.set_matching_cliente("".into());
            v.set_matching_asesor("".into());
            v.set_matching_tipo_operacion("Venta".into());
            v.set_matching_zona("".into());
            v.set_matching_presupuesto("".into());
            v.set_matching_inmueble("".into());
            v.set_matching_precio("".into());
            v.set_matching_fecha_venc_anio(y);
            v.set_matching_fecha_venc_mes(m);
            v.set_matching_fecha_venc_dia(d);
            v.set_matching_editando(-1);
            v.set_mensaje_matching("".into());
            v.set_mostrar_form_matching(true);
        }
    });
    let debil = v.as_weak();
    v.on_nuevo_legal(move || {
        if let Some(v) = debil.upgrade() {
            v.set_legal_cod_inmueble("".into());
            v.set_legal_propietario("".into());
            v.set_legal_titulo(false);
            v.set_legal_cedula(false);
            v.set_legal_catastral(false);
            v.set_legal_solvencia(false);
            v.set_legal_liberacion_hipoteca("N/A".into());
            v.set_legal_borrador_contrato("PENDIENTE".into());
            v.set_legal_estatus_notaria("PENDIENTE".into());
            v.set_legal_editando(-1);
            v.set_mensaje_legal("".into());
            v.set_mostrar_form_legal(true);
        }
    });
    let debil = v.as_weak();
    v.on_nuevo_embudo(move || {
        if let Some(v) = debil.upgrade() {
            v.set_embudo_id_asesor("".into());
            v.set_embudo_nombre_asesor("".into());
            v.set_embudo_llamadas("".into());
            v.set_embudo_citas("".into());
            v.set_embudo_visitas("".into());
            v.set_embudo_ofertas("".into());
            v.set_embudo_cierres("".into());
            v.set_embudo_editando(-1);
            v.set_mensaje_embudo("".into());
            v.set_mostrar_form_embudo(true);
        }
    });
    let debil = v.as_weak();
    v.on_nuevo_cierre(move || {
        if let Some(v) = debil.upgrade() {
            let (y, m, d) = hoy_dma();
            v.set_cierre_id("".into());
            v.set_cierre_fecha_anio(y);
            v.set_cierre_fecha_mes(m);
            v.set_cierre_fecha_dia(d);
            v.set_cierre_cod_inmueble("".into());
            v.set_cierre_tipo_operacion("Venta".into());
            v.set_cierre_monto("".into());
            v.set_cierre_id_captador("".into());
            v.set_cierre_id_cerrador("".into());
            v.set_cierre_pct_comision("".into());
            v.set_cierre_editando(-1);
            v.set_mensaje_cierre("".into());
            v.set_mostrar_form_cierre(true);
        }
    });
    let debil = v.as_weak();
    v.on_nuevo_captacion(move || {
        if let Some(v) = debil.upgrade() {
            let (y, m, d) = hoy_dma();
            v.set_captacion_cod_inmueble("".into());
            v.set_captacion_tipo_propiedad("Casa".into());
            v.set_captacion_id_captador("".into());
            v.set_captacion_fecha_anio(y);
            v.set_captacion_fecha_mes(m);
            v.set_captacion_fecha_dia(d);
            v.set_captacion_precio("".into());
            v.set_captacion_estatus("Disponible".into());
            v.set_captacion_web(false);
            v.set_captacion_rrss(false);
            v.set_captacion_editando(-1);
            v.set_mensaje_captacion("".into());
            v.set_mostrar_form_captacion(true);
        }
    });
    let debil = v.as_weak();
    v.on_nuevo_finanza(move || {
        if let Some(v) = debil.upgrade() {
            let (y, m, d) = hoy_dma();
            v.set_finanza_id("".into());
            v.set_finanza_fecha_anio(y);
            v.set_finanza_fecha_mes(m);
            v.set_finanza_fecha_dia(d);
            v.set_finanza_semana("".into());
            v.set_finanza_tipo_flujo("Ingreso".into());
            v.set_finanza_categoria("".into());
            v.set_finanza_monto("".into());
            v.set_finanza_estatus("Pendiente".into());
            v.set_finanza_editando(-1);
            v.set_mensaje_finanza("".into());
            v.set_mostrar_form_finanza(true);
        }
    });
    let debil = v.as_weak();
    v.on_nuevo_reporte(move || {
        if let Some(v) = debil.upgrade() {
            v.set_reporte_cod_inmueble("".into());
            v.set_reporte_propietario("".into());
            v.set_reporte_telefono("".into());
            v.set_reporte_id_asesor("".into());
            v.set_reporte_visitas("".into());
            v.set_reporte_ofertas("".into());
            v.set_reporte_canales("".into());
            v.set_reporte_notas("".into());
            v.set_reporte_estatus_envio("PENDIENTE".into());
            v.set_reporte_editando(-1);
            v.set_mensaje_reporte("".into());
            v.set_mostrar_form_reporte(true);
        }
    });

    // --- Alta / edición: Matching ---
    let debil = v.as_weak();
    v.on_editar_matching(move |i| {
        if let Some(v) = debil.upgrade() {
            let db = data::cargar();
            if let Some(r) = db.matching.get(i as usize) {
                v.set_matching_id(r.id.clone().into());
                v.set_matching_cliente(r.cliente_buscador.clone().into());
                v.set_matching_asesor(r.asesor_cliente.clone().into());
                v.set_matching_tipo_operacion(r.tipo_operacion.clone().into());
                v.set_matching_zona(r.zona_deseada.clone().into());
                v.set_matching_presupuesto(r.presupuesto_max.to_string().into());
                v.set_matching_inmueble(r.inmueble_matcheado.clone().into());
                v.set_matching_precio(r.precio_lista.to_string().into());
                let (y, m, d) = partes_de_fecha(&r.fecha_venc_exclusividad);
                v.set_matching_fecha_venc_anio(y);
                v.set_matching_fecha_venc_mes(m);
                v.set_matching_fecha_venc_dia(d);
                v.set_matching_editando(i);
                v.set_mostrar_form_matching(true);
                v.set_mensaje_matching("".into());
            }
        }
    });
    let debil = v.as_weak();
    v.on_guardar_matching(move || {
        let v = match debil.upgrade() {
            Some(v) => v,
            None => return,
        };
        let id = v.get_matching_id().to_string();
        if id.trim().is_empty() {
            v.set_mensaje_matching("Falta el ID".into());
            return;
        }
        let asesor = v.get_matching_asesor().to_string();
        let mut db = data::cargar();
        if !asesor.trim().is_empty() && !existe_asesor(&db, &asesor) {
            v.set_mensaje_matching(format!("El asesor '{asesor}' no existe en BD Asesores").into());
            return;
        }
        let presupuesto = numero_obligatorio!(v, v.get_matching_presupuesto().to_string(), "Presupuesto máx.", set_mensaje_matching);
        let precio = numero_obligatorio!(v, v.get_matching_precio().to_string(), "Precio lista inmueble", set_mensaje_matching);
        let idx = v.get_matching_editando();
        let registro = Requerimiento {
            id,
            cliente_buscador: v.get_matching_cliente().to_string(),
            asesor_cliente: asesor,
            tipo_operacion: v.get_matching_tipo_operacion().to_string(),
            zona_deseada: v.get_matching_zona().to_string(),
            presupuesto_max: presupuesto,
            inmueble_matcheado: v.get_matching_inmueble().to_string(),
            precio_lista: precio,
            fecha_venc_exclusividad: fecha_str(v.get_matching_fecha_venc_anio(), v.get_matching_fecha_venc_mes(), v.get_matching_fecha_venc_dia()),
        };
        if idx >= 0 && (idx as usize) < db.matching.len() {
            db.matching[idx as usize] = registro;
        } else {
            db.matching.push(registro);
        }
        if let Err(e) = data::guardar(&db) {
            v.set_mensaje_matching(format!("Error al guardar: {e}").into());
            return;
        }
        v.set_matching_id("".into());
        v.set_matching_cliente("".into());
        v.set_matching_asesor("".into());
        v.set_matching_tipo_operacion("".into());
        v.set_matching_zona("".into());
        v.set_matching_presupuesto("".into());
        v.set_matching_inmueble("".into());
        v.set_matching_precio("".into());
        v.set_matching_editando(-1);
        v.set_mostrar_form_matching(false);
        v.set_mensaje_matching("Guardado ✓".into());
        cargar_matching(&v);
    });

    // --- Alta / edición: Legal ---
    let debil = v.as_weak();
    v.on_editar_legal(move |i| {
        if let Some(v) = debil.upgrade() {
            let db = data::cargar();
            if let Some(e) = db.legal.get(i as usize) {
                v.set_legal_cod_inmueble(e.cod_inmueble.clone().into());
                v.set_legal_propietario(e.propietario.clone().into());
                v.set_legal_titulo(e.titulo_propiedad);
                v.set_legal_cedula(e.cedula_rif);
                v.set_legal_catastral(e.ficha_catastral);
                v.set_legal_solvencia(e.solvencia_municipal);
                v.set_legal_liberacion_hipoteca(e.liberacion_hipoteca.clone().into());
                v.set_legal_borrador_contrato(e.borrador_contrato.clone().into());
                v.set_legal_estatus_notaria(e.estatus_notaria.clone().into());
                v.set_legal_editando(i);
                v.set_mostrar_form_legal(true);
                v.set_mensaje_legal("".into());
            }
        }
    });
    let debil = v.as_weak();
    v.on_guardar_legal(move || {
        let v = match debil.upgrade() {
            Some(v) => v,
            None => return,
        };
        let cod = v.get_legal_cod_inmueble().to_string();
        if cod.trim().is_empty() {
            v.set_mensaje_legal("Falta el código de inmueble".into());
            return;
        }
        let idx = v.get_legal_editando();
        let mut db = data::cargar();
        let registro = ExpedienteLegal {
            cod_inmueble: cod,
            propietario: v.get_legal_propietario().to_string(),
            titulo_propiedad: v.get_legal_titulo(),
            cedula_rif: v.get_legal_cedula(),
            ficha_catastral: v.get_legal_catastral(),
            solvencia_municipal: v.get_legal_solvencia(),
            liberacion_hipoteca: v.get_legal_liberacion_hipoteca().to_string(),
            borrador_contrato: v.get_legal_borrador_contrato().to_string(),
            estatus_notaria: v.get_legal_estatus_notaria().to_string(),
        };
        if idx >= 0 && (idx as usize) < db.legal.len() {
            db.legal[idx as usize] = registro;
        } else {
            db.legal.push(registro);
        }
        if let Err(e) = data::guardar(&db) {
            v.set_mensaje_legal(format!("Error al guardar: {e}").into());
            return;
        }
        v.set_legal_cod_inmueble("".into());
        v.set_legal_propietario("".into());
        v.set_legal_titulo(false);
        v.set_legal_cedula(false);
        v.set_legal_catastral(false);
        v.set_legal_solvencia(false);
        v.set_legal_liberacion_hipoteca("".into());
        v.set_legal_borrador_contrato("".into());
        v.set_legal_estatus_notaria("".into());
        v.set_legal_editando(-1);
        v.set_mostrar_form_legal(false);
        v.set_mensaje_legal("Guardado ✓".into());
        cargar_legal(&v);
    });

    // --- Alta / edición: Embudo ---
    let debil = v.as_weak();
    v.on_editar_embudo(move |i| {
        if let Some(v) = debil.upgrade() {
            let db = data::cargar();
            if let Some(e) = db.embudo.get(i as usize) {
                v.set_embudo_id_asesor(e.id_asesor.clone().into());
                v.set_embudo_nombre_asesor(e.nombre_asesor.clone().into());
                v.set_embudo_llamadas(e.llamadas_realizadas.to_string().into());
                v.set_embudo_citas(e.citas_captacion.to_string().into());
                v.set_embudo_visitas(e.visitas_guiadas.to_string().into());
                v.set_embudo_ofertas(e.ofertas_recibidas.to_string().into());
                v.set_embudo_cierres(e.cierres_mes.to_string().into());
                v.set_embudo_editando(i);
                v.set_mostrar_form_embudo(true);
                v.set_mensaje_embudo("".into());
            }
        }
    });
    let debil = v.as_weak();
    v.on_guardar_embudo(move || {
        let v = match debil.upgrade() {
            Some(v) => v,
            None => return,
        };
        let id = v.get_embudo_id_asesor().to_string();
        if id.trim().is_empty() {
            v.set_mensaje_embudo("Falta el ID de asesor".into());
            return;
        }
        let mut db = data::cargar();
        if !existe_asesor(&db, &id) {
            v.set_mensaje_embudo(format!("El asesor '{id}' no existe en BD Asesores").into());
            return;
        }
        let llamadas = numero_obligatorio!(v, v.get_embudo_llamadas().to_string(), "Llamadas realizadas", set_mensaje_embudo);
        let citas = numero_obligatorio!(v, v.get_embudo_citas().to_string(), "Citas captación", set_mensaje_embudo);
        let visitas = numero_obligatorio!(v, v.get_embudo_visitas().to_string(), "Visitas guiadas", set_mensaje_embudo);
        let ofertas = numero_obligatorio!(v, v.get_embudo_ofertas().to_string(), "Ofertas recibidas", set_mensaje_embudo);
        let cierres = numero_obligatorio!(v, v.get_embudo_cierres().to_string(), "Cierres del mes", set_mensaje_embudo);
        let idx = v.get_embudo_editando();
        let registro = EmbudoAsesor {
            id_asesor: id,
            nombre_asesor: v.get_embudo_nombre_asesor().to_string(),
            llamadas_realizadas: llamadas,
            citas_captacion: citas,
            visitas_guiadas: visitas,
            ofertas_recibidas: ofertas,
            cierres_mes: cierres,
        };
        if idx >= 0 && (idx as usize) < db.embudo.len() {
            db.embudo[idx as usize] = registro;
        } else {
            db.embudo.push(registro);
        }
        if let Err(e) = data::guardar(&db) {
            v.set_mensaje_embudo(format!("Error al guardar: {e}").into());
            return;
        }
        v.set_embudo_id_asesor("".into());
        v.set_embudo_nombre_asesor("".into());
        v.set_embudo_llamadas("".into());
        v.set_embudo_citas("".into());
        v.set_embudo_visitas("".into());
        v.set_embudo_ofertas("".into());
        v.set_embudo_cierres("".into());
        v.set_embudo_editando(-1);
        v.set_mostrar_form_embudo(false);
        v.set_mensaje_embudo("Guardado ✓".into());
        cargar_embudo(&v);
    });

    // --- Alta / edición: Reporte Propietario ---
    let debil = v.as_weak();
    v.on_editar_reporte(move |i| {
        if let Some(v) = debil.upgrade() {
            let db = data::cargar();
            if let Some(r) = db.reportes.get(i as usize) {
                v.set_reporte_cod_inmueble(r.cod_inmueble.clone().into());
                v.set_reporte_propietario(r.propietario.clone().into());
                v.set_reporte_telefono(r.telefono.clone().into());
                v.set_reporte_id_asesor(r.id_asesor.clone().into());
                v.set_reporte_visitas(r.visitas_agendadas.to_string().into());
                v.set_reporte_ofertas(r.ofertas_recibidas.to_string().into());
                v.set_reporte_canales(r.canales_publicacion.clone().into());
                v.set_reporte_notas(r.notas.clone().into());
                v.set_reporte_estatus_envio(r.estatus_envio.clone().into());
                v.set_reporte_editando(i);
                v.set_mostrar_form_reporte(true);
                v.set_mensaje_reporte("".into());
            }
        }
    });
    let debil = v.as_weak();
    v.on_guardar_reporte(move || {
        let v = match debil.upgrade() {
            Some(v) => v,
            None => return,
        };
        let cod = v.get_reporte_cod_inmueble().to_string();
        if cod.trim().is_empty() {
            v.set_mensaje_reporte("Falta el código de inmueble".into());
            return;
        }
        let id_asesor = v.get_reporte_id_asesor().to_string();
        let mut db = data::cargar();
        if !id_asesor.trim().is_empty() && !existe_asesor(&db, &id_asesor) {
            v.set_mensaje_reporte(format!("El asesor '{id_asesor}' no existe en BD Asesores").into());
            return;
        }
        let idx = v.get_reporte_editando();
        let visitas = numero_obligatorio!(v, v.get_reporte_visitas().to_string(), "Visitas agendadas", set_mensaje_reporte);
        let ofertas = numero_obligatorio!(v, v.get_reporte_ofertas().to_string(), "Ofertas recibidas", set_mensaje_reporte);
        let registro = ReportePropietario {
            cod_inmueble: cod,
            propietario: v.get_reporte_propietario().to_string(),
            telefono: v.get_reporte_telefono().to_string(),
            id_asesor,
            visitas_agendadas: visitas,
            ofertas_recibidas: ofertas,
            canales_publicacion: v.get_reporte_canales().to_string(),
            notas: v.get_reporte_notas().to_string(),
            estatus_envio: v.get_reporte_estatus_envio().to_string(),
        };
        if idx >= 0 && (idx as usize) < db.reportes.len() {
            db.reportes[idx as usize] = registro;
        } else {
            db.reportes.push(registro);
        }
        if let Err(e) = data::guardar(&db) {
            v.set_mensaje_reporte(format!("Error al guardar: {e}").into());
            return;
        }
        v.set_reporte_cod_inmueble("".into());
        v.set_reporte_propietario("".into());
        v.set_reporte_telefono("".into());
        v.set_reporte_id_asesor("".into());
        v.set_reporte_visitas("".into());
        v.set_reporte_ofertas("".into());
        v.set_reporte_canales("".into());
        v.set_reporte_notas("".into());
        v.set_reporte_estatus_envio("".into());
        v.set_reporte_editando(-1);
        v.set_mostrar_form_reporte(false);
        v.set_mensaje_reporte("Guardado ✓".into());
        cargar_reportes(&v);
    });

    // --- Alta / edición: Cierre ---
    let debil = v.as_weak();
    v.on_editar_cierre(move |i| {
        if let Some(v) = debil.upgrade() {
            let db = data::cargar();
            if let Some(c) = db.cierres.get(i as usize) {
                v.set_cierre_id(c.id.clone().into());
                let (y, m, d) = partes_de_fecha(&c.fecha_cierre);
                v.set_cierre_fecha_anio(y);
                v.set_cierre_fecha_mes(m);
                v.set_cierre_fecha_dia(d);
                v.set_cierre_cod_inmueble(c.cod_inmueble.clone().into());
                v.set_cierre_tipo_operacion(c.tipo_operacion.clone().into());
                v.set_cierre_monto(c.monto_operacion.to_string().into());
                v.set_cierre_id_captador(c.id_captador.clone().into());
                v.set_cierre_id_cerrador(c.id_cerrador.clone().into());
                v.set_cierre_pct_comision(c.pct_comision_total.to_string().into());
                v.set_cierre_editando(i);
                v.set_mostrar_form_cierre(true);
                v.set_mensaje_cierre("".into());
            }
        }
    });
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
        let id_captador = v.get_cierre_id_captador().to_string();
        let id_cerrador = v.get_cierre_id_cerrador().to_string();
        let mut db = data::cargar();
        if !id_captador.trim().is_empty() && !existe_asesor(&db, &id_captador) {
            v.set_mensaje_cierre(format!("El captador '{id_captador}' no existe en BD Asesores").into());
            return;
        }
        if !id_cerrador.trim().is_empty() && !existe_asesor(&db, &id_cerrador) {
            v.set_mensaje_cierre(format!("El cerrador '{id_cerrador}' no existe en BD Asesores").into());
            return;
        }
        let idx = v.get_cierre_editando();
        let monto = numero_obligatorio!(v, v.get_cierre_monto().to_string(), "Monto operación", set_mensaje_cierre);
        let pct = numero_obligatorio!(v, v.get_cierre_pct_comision().to_string(), "% Comisión", set_mensaje_cierre);
        let registro = Cierre {
            id,
            fecha_cierre: fecha_str(v.get_cierre_fecha_anio(), v.get_cierre_fecha_mes(), v.get_cierre_fecha_dia()),
            cod_inmueble: v.get_cierre_cod_inmueble().to_string(),
            tipo_operacion: v.get_cierre_tipo_operacion().to_string(),
            monto_operacion: monto,
            id_captador,
            id_cerrador,
            pct_comision_total: pct,
        };
        if idx >= 0 && (idx as usize) < db.cierres.len() {
            db.cierres[idx as usize] = registro;
        } else {
            db.cierres.push(registro);
        }
        if let Err(e) = data::guardar(&db) {
            v.set_mensaje_cierre(format!("Error al guardar: {e}").into());
            return;
        }
        v.set_cierre_id("".into());
        v.set_cierre_cod_inmueble("".into());
        v.set_cierre_monto("".into());
        v.set_cierre_id_captador("".into());
        v.set_cierre_id_cerrador("".into());
        v.set_cierre_pct_comision("".into());
        v.set_cierre_editando(-1);
        v.set_mostrar_form_cierre(false);
        v.set_mensaje_cierre("Guardado ✓".into());
        cargar_cierres(&v);
        cargar_dashboard(&v);
        cargar_asesores(&v);
    });

    // --- Eliminar filas (borra por indice, el mismo orden en que se listan) ---
    macro_rules! conectar_eliminar {
        ($cb:ident, $campo:ident, $recargar:ident, $set_editando:ident) => {
            let debil = v.as_weak();
            v.$cb(move |i| {
                if let Some(v) = debil.upgrade() {
                    let mut db = data::cargar();
                    if (i as usize) < db.$campo.len() {
                        db.$campo.remove(i as usize);
                        if let Err(e) = data::guardar(&db) {
                            eprintln!("Error al guardar tras eliminar: {e}");
                        }
                    }
                    v.$set_editando(-1);
                    $recargar(&v);
                }
            });
        };
    }
    conectar_eliminar!(on_eliminar_asesor, asesores, cargar_asesores, set_asesor_editando);
    conectar_eliminar!(on_eliminar_matching, matching, cargar_matching, set_matching_editando);
    conectar_eliminar!(on_eliminar_legal, legal, cargar_legal, set_legal_editando);
    conectar_eliminar!(on_eliminar_embudo, embudo, cargar_embudo, set_embudo_editando);
    conectar_eliminar!(on_eliminar_captacion, captaciones, cargar_captaciones, set_captacion_editando);
    conectar_eliminar!(on_eliminar_reporte, reportes, cargar_reportes, set_reporte_editando);

    // Cierres y Finanzas alimentan el Dashboard, asi que al eliminar uno
    // tambien se refresca el resumen.
    let debil = v.as_weak();
    v.on_eliminar_cierre(move |i| {
        if let Some(v) = debil.upgrade() {
            let mut db = data::cargar();
            if (i as usize) < db.cierres.len() {
                db.cierres.remove(i as usize);
                if let Err(e) = data::guardar(&db) {
                    eprintln!("Error al guardar tras eliminar: {e}");
                }
            }
            v.set_cierre_editando(-1);
            cargar_cierres(&v);
            cargar_dashboard(&v);
        }
    });

    let debil = v.as_weak();
    v.on_eliminar_finanza(move |i| {
        if let Some(v) = debil.upgrade() {
            let mut db = data::cargar();
            if (i as usize) < db.finanzas.len() {
                db.finanzas.remove(i as usize);
                if let Err(e) = data::guardar(&db) {
                    eprintln!("Error al guardar tras eliminar: {e}");
                }
            }
            v.set_finanza_editando(-1);
            cargar_finanzas(&v);
            cargar_dashboard(&v);
        }
    });

    // --- Alta / edición: Asesor ---
    let debil = v.as_weak();
    v.on_editar_asesor(move |i| {
        if let Some(v) = debil.upgrade() {
            let db = data::cargar();
            if let Some(a) = db.asesores.get(i as usize) {
                v.set_asesor_id(a.id.clone().into());
                v.set_asesor_nombre(a.nombre.clone().into());
                let (y, m, d) = partes_de_fecha(&a.fecha_ingreso);
                v.set_asesor_fecha_ingreso_anio(y);
                v.set_asesor_fecha_ingreso_mes(m);
                v.set_asesor_fecha_ingreso_dia(d);
                v.set_asesor_talleres(a.talleres_asistidos.to_string().into());
                v.set_asesor_editando(i);
                v.set_mostrar_form_asesor(true);
                v.set_mensaje_asesor("".into());
            }
        }
    });
    let debil = v.as_weak();
    v.on_guardar_asesor(move || {
        let v = match debil.upgrade() {
            Some(v) => v,
            None => return,
        };
        let id = v.get_asesor_id().to_string();
        if id.trim().is_empty() {
            v.set_mensaje_asesor("Falta el ID".into());
            return;
        }
        let idx = v.get_asesor_editando();
        let mut db = data::cargar();
        let duplicado = db
            .asesores
            .iter()
            .enumerate()
            .any(|(pos, a)| a.id == id && pos as i32 != idx);
        if duplicado {
            v.set_mensaje_asesor("Ya existe un asesor con ese ID".into());
            return;
        }
        let talleres = numero_obligatorio!(v, v.get_asesor_talleres().to_string(), "Talleres asistidos", set_mensaje_asesor);
        let registro = Asesor {
            id,
            nombre: v.get_asesor_nombre().to_string(),
            fecha_ingreso: fecha_str(v.get_asesor_fecha_ingreso_anio(), v.get_asesor_fecha_ingreso_mes(), v.get_asesor_fecha_ingreso_dia()),
            talleres_asistidos: talleres,
        };
        if idx >= 0 && (idx as usize) < db.asesores.len() {
            db.asesores[idx as usize] = registro;
        } else {
            db.asesores.push(registro);
        }
        if let Err(e) = data::guardar(&db) {
            v.set_mensaje_asesor(format!("Error al guardar: {e}").into());
            return;
        }
        v.set_asesor_id("".into());
        v.set_asesor_nombre("".into());
        v.set_asesor_talleres("".into());
        v.set_asesor_editando(-1);
        v.set_mostrar_form_asesor(false);
        v.set_mensaje_asesor("Guardado ✓".into());
        cargar_asesores(&v);
    });

    // --- Alta / edición: Captación ---
    let debil = v.as_weak();
    v.on_editar_captacion(move |i| {
        if let Some(v) = debil.upgrade() {
            let db = data::cargar();
            if let Some(c) = db.captaciones.get(i as usize) {
                v.set_captacion_cod_inmueble(c.cod_inmueble.clone().into());
                v.set_captacion_tipo_propiedad(c.tipo_propiedad.clone().into());
                v.set_captacion_id_captador(c.id_captador.clone().into());
                let (y, m, d) = partes_de_fecha(&c.fecha_captacion);
                v.set_captacion_fecha_anio(y);
                v.set_captacion_fecha_mes(m);
                v.set_captacion_fecha_dia(d);
                v.set_captacion_precio(c.precio_lista.to_string().into());
                v.set_captacion_estatus(c.estatus.clone().into());
                v.set_captacion_web(c.publicado_web);
                v.set_captacion_rrss(c.publicado_rrss);
                v.set_captacion_editando(i);
                v.set_mostrar_form_captacion(true);
                v.set_mensaje_captacion("".into());
            }
        }
    });
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
        let id_captador = v.get_captacion_id_captador().to_string();
        let mut db = data::cargar();
        if !id_captador.trim().is_empty() && !existe_asesor(&db, &id_captador) {
            v.set_mensaje_captacion(format!("El captador '{id_captador}' no existe en BD Asesores").into());
            return;
        }
        let idx = v.get_captacion_editando();
        let precio = numero_obligatorio!(v, v.get_captacion_precio().to_string(), "Precio Lista", set_mensaje_captacion);
        let registro = Captacion {
            cod_inmueble: cod,
            tipo_propiedad: v.get_captacion_tipo_propiedad().to_string(),
            id_captador,
            fecha_captacion: fecha_str(v.get_captacion_fecha_anio(), v.get_captacion_fecha_mes(), v.get_captacion_fecha_dia()),
            precio_lista: precio,
            estatus: v.get_captacion_estatus().to_string(),
            publicado_web: v.get_captacion_web(),
            publicado_rrss: v.get_captacion_rrss(),
        };
        if idx >= 0 && (idx as usize) < db.captaciones.len() {
            db.captaciones[idx as usize] = registro;
        } else {
            db.captaciones.push(registro);
        }
        if let Err(e) = data::guardar(&db) {
            v.set_mensaje_captacion(format!("Error al guardar: {e}").into());
            return;
        }
        v.set_captacion_cod_inmueble("".into());
        v.set_captacion_id_captador("".into());
        v.set_captacion_precio("".into());
        v.set_captacion_web(false);
        v.set_captacion_rrss(false);
        v.set_captacion_editando(-1);
        v.set_mostrar_form_captacion(false);
        v.set_mensaje_captacion("Guardado ✓".into());
        cargar_captaciones(&v);
    });

    // --- Alta / edición: Finanza ---
    let debil = v.as_weak();
    v.on_editar_finanza(move |i| {
        if let Some(v) = debil.upgrade() {
            let db = data::cargar();
            if let Some(t) = db.finanzas.get(i as usize) {
                v.set_finanza_id(t.id.clone().into());
                let (y, m, d) = partes_de_fecha(&t.fecha);
                v.set_finanza_fecha_anio(y);
                v.set_finanza_fecha_mes(m);
                v.set_finanza_fecha_dia(d);
                v.set_finanza_semana(t.semana.clone().into());
                v.set_finanza_tipo_flujo(t.tipo_flujo.clone().into());
                v.set_finanza_categoria(t.categoria.clone().into());
                v.set_finanza_monto(t.monto.to_string().into());
                v.set_finanza_estatus(t.estatus_pago.clone().into());
                v.set_finanza_editando(i);
                v.set_mostrar_form_finanza(true);
                v.set_mensaje_finanza("".into());
            }
        }
    });
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
        let idx = v.get_finanza_editando();
        let mut db = data::cargar();
        let monto = numero_obligatorio!(v, v.get_finanza_monto().to_string(), "Monto", set_mensaje_finanza);
        let registro = TransaccionFinanciera {
            id,
            fecha: fecha_str(v.get_finanza_fecha_anio(), v.get_finanza_fecha_mes(), v.get_finanza_fecha_dia()),
            semana: v.get_finanza_semana().to_string(),
            tipo_flujo: v.get_finanza_tipo_flujo().to_string(),
            categoria: v.get_finanza_categoria().to_string(),
            monto,
            estatus_pago: v.get_finanza_estatus().to_string(),
        };
        if idx >= 0 && (idx as usize) < db.finanzas.len() {
            db.finanzas[idx as usize] = registro;
        } else {
            db.finanzas.push(registro);
        }
        if let Err(e) = data::guardar(&db) {
            v.set_mensaje_finanza(format!("Error al guardar: {e}").into());
            return;
        }
        v.set_finanza_id("".into());
        v.set_finanza_semana("".into());
        v.set_finanza_categoria("".into());
        v.set_finanza_monto("".into());
        v.set_finanza_editando(-1);
        v.set_mostrar_form_finanza(false);
        v.set_mensaje_finanza("Guardado ✓".into());
        cargar_finanzas(&v);
        cargar_dashboard(&v);
    });

    v.run().expect("error al correr la app Slint");
}
