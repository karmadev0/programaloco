// ============================================================================
// data.rs
// Carga y guarda cada hoja como un .csv dentro de ./data/
// Formato CSV simple (sin comillas ni comas dentro de campos) para no
// necesitar ninguna dependencia externa. Alcanza de sobra para este caso.
// ============================================================================

use crate::models::*;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

// ----------------------------------------------------------------------
// EXTENSION (no toca la logica original): la carpeta de datos ya no es
// una ruta relativa fija ("./data"), porque en una app instalada
// (Program Files, /Applications, etc.) esa carpeta puede no ser
// escribible o puede no existir el cwd esperado.
// inicializar_dir() se llama UNA vez al arrancar Tauri, con la carpeta
// de datos de usuario que da el sistema operativo (app_data_dir).
// Si nadie la llama (ej. tests o el binario viejo de terminal), se cae
// al comportamiento original: "./data" relativo al cwd.
// ----------------------------------------------------------------------
static BASE_DIR: OnceLock<PathBuf> = OnceLock::new();

pub fn inicializar_dir(dir: PathBuf) {
    let _ = BASE_DIR.set(dir);
}

fn base_dir() -> String {
    match BASE_DIR.get() {
        Some(p) => p.to_string_lossy().to_string(),
        None => "data".to_string(),
    }
}

fn ruta(nombre: &str) -> String {
    format!("{}/{}.csv", base_dir(), nombre)
}

/// Lee un CSV y devuelve las filas ya separadas por columna (se salta la
/// primera linea, que es el encabezado).
fn leer_filas(nombre: &str) -> Vec<Vec<String>> {
    let path = ruta(nombre);
    let contenido = match fs::read_to_string(&path) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };
    contenido
        .lines()
        .skip(1)
        .filter(|l| !l.trim().is_empty())
        .map(|l| l.split(',').map(|s| s.trim().to_string()).collect())
        .collect()
}

fn escribir_csv(nombre: &str, encabezado: &str, filas: &[String]) -> std::io::Result<()> {
    fs::create_dir_all(base_dir())?;
    let mut f = fs::File::create(ruta(nombre))?;
    writeln!(f, "{}", encabezado)?;
    for fila in filas {
        writeln!(f, "{}", fila)?;
    }
    Ok(())
}

fn pf(s: &str) -> f64 {
    s.trim().parse::<f64>().unwrap_or(0.0)
}
fn pb(s: &str) -> bool {
    matches!(s.trim().to_uppercase().as_str(), "SI" | "SÍ" | "TRUE" | "1")
}
fn bf(b: bool) -> &'static str {
    if b { "SI" } else { "NO" }
}

pub fn cargar() -> Database {
    let asesores = leer_filas("asesores")
        .into_iter()
        .filter(|r| r.len() >= 4)
        .map(|r| Asesor {
            id: r[0].clone(),
            nombre: r[1].clone(),
            fecha_ingreso: r[2].clone(),
            talleres_asistidos: pf(&r[3]),
        })
        .collect();

    let captaciones = leer_filas("captaciones")
        .into_iter()
        .filter(|r| r.len() >= 8)
        .map(|r| Captacion {
            cod_inmueble: r[0].clone(),
            tipo_propiedad: r[1].clone(),
            id_captador: r[2].clone(),
            fecha_captacion: r[3].clone(),
            precio_lista: pf(&r[4]),
            estatus: r[5].clone(),
            publicado_web: pb(&r[6]),
            publicado_rrss: pb(&r[7]),
        })
        .collect();

    let cierres = leer_filas("cierres")
        .into_iter()
        .filter(|r| r.len() >= 8)
        .map(|r| Cierre {
            id: r[0].clone(),
            fecha_cierre: r[1].clone(),
            cod_inmueble: r[2].clone(),
            tipo_operacion: r[3].clone(),
            monto_operacion: pf(&r[4]),
            id_captador: r[5].clone(),
            id_cerrador: r[6].clone(),
            pct_comision_total: pf(&r[7]),
        })
        .collect();

    let finanzas = leer_filas("finanzas")
        .into_iter()
        .filter(|r| r.len() >= 7)
        .map(|r| TransaccionFinanciera {
            id: r[0].clone(),
            fecha: r[1].clone(),
            semana: r[2].clone(),
            tipo_flujo: r[3].clone(),
            categoria: r[4].clone(),
            monto: pf(&r[5]),
            estatus_pago: r[6].clone(),
        })
        .collect();

    let matching = leer_filas("matching")
        .into_iter()
        .filter(|r| r.len() >= 9)
        .map(|r| Requerimiento {
            id: r[0].clone(),
            cliente_buscador: r[1].clone(),
            asesor_cliente: r[2].clone(),
            tipo_operacion: r[3].clone(),
            zona_deseada: r[4].clone(),
            presupuesto_max: pf(&r[5]),
            inmueble_matcheado: r[6].clone(),
            precio_lista: pf(&r[7]),
            fecha_venc_exclusividad: r[8].clone(),
        })
        .collect();

    let legal = leer_filas("legal")
        .into_iter()
        .filter(|r| r.len() >= 9)
        .map(|r| ExpedienteLegal {
            cod_inmueble: r[0].clone(),
            propietario: r[1].clone(),
            titulo_propiedad: pb(&r[2]),
            cedula_rif: pb(&r[3]),
            ficha_catastral: pb(&r[4]),
            solvencia_municipal: pb(&r[5]),
            liberacion_hipoteca: r[6].clone(),
            borrador_contrato: r[7].clone(),
            estatus_notaria: r[8].clone(),
        })
        .collect();

    let embudo = leer_filas("embudo")
        .into_iter()
        .filter(|r| r.len() >= 7)
        .map(|r| EmbudoAsesor {
            id_asesor: r[0].clone(),
            nombre_asesor: r[1].clone(),
            llamadas_realizadas: pf(&r[2]),
            citas_captacion: pf(&r[3]),
            visitas_guiadas: pf(&r[4]),
            ofertas_recibidas: pf(&r[5]),
            cierres_mes: pf(&r[6]),
        })
        .collect();

    let reportes = leer_filas("reportes")
        .into_iter()
        .filter(|r| r.len() >= 9)
        .map(|r| ReportePropietario {
            cod_inmueble: r[0].clone(),
            propietario: r[1].clone(),
            telefono: r[2].clone(),
            id_asesor: r[3].clone(),
            visitas_agendadas: pf(&r[4]),
            ofertas_recibidas: pf(&r[5]),
            canales_publicacion: r[6].clone(),
            notas: r[7].clone(),
            estatus_envio: r[8].clone(),
        })
        .collect();

    Database {
        asesores,
        captaciones,
        cierres,
        finanzas,
        matching,
        legal,
        embudo,
        reportes,
    }
}

pub fn guardar(db: &Database) -> std::io::Result<()> {
    let filas: Vec<String> = db
        .asesores
        .iter()
        .map(|a| format!("{},{},{},{}", a.id, a.nombre, a.fecha_ingreso, a.talleres_asistidos))
        .collect();
    escribir_csv("asesores", "id,nombre,fecha_ingreso,talleres_asistidos", &filas)?;

    let filas: Vec<String> = db
        .captaciones
        .iter()
        .map(|c| {
            format!(
                "{},{},{},{},{},{},{},{}",
                c.cod_inmueble,
                c.tipo_propiedad,
                c.id_captador,
                c.fecha_captacion,
                c.precio_lista,
                c.estatus,
                bf(c.publicado_web),
                bf(c.publicado_rrss)
            )
        })
        .collect();
    escribir_csv(
        "captaciones",
        "cod_inmueble,tipo_propiedad,id_captador,fecha_captacion,precio_lista,estatus,publicado_web,publicado_rrss",
        &filas,
    )?;

    let filas: Vec<String> = db
        .cierres
        .iter()
        .map(|c| {
            format!(
                "{},{},{},{},{},{},{},{}",
                c.id,
                c.fecha_cierre,
                c.cod_inmueble,
                c.tipo_operacion,
                c.monto_operacion,
                c.id_captador,
                c.id_cerrador,
                c.pct_comision_total
            )
        })
        .collect();
    escribir_csv(
        "cierres",
        "id,fecha_cierre,cod_inmueble,tipo_operacion,monto_operacion,id_captador,id_cerrador,pct_comision_total",
        &filas,
    )?;

    let filas: Vec<String> = db
        .finanzas
        .iter()
        .map(|t| {
            format!(
                "{},{},{},{},{},{},{}",
                t.id, t.fecha, t.semana, t.tipo_flujo, t.categoria, t.monto, t.estatus_pago
            )
        })
        .collect();
    escribir_csv("finanzas", "id,fecha,semana,tipo_flujo,categoria,monto,estatus_pago", &filas)?;

    let filas: Vec<String> = db
        .matching
        .iter()
        .map(|r| {
            format!(
                "{},{},{},{},{},{},{},{},{}",
                r.id,
                r.cliente_buscador,
                r.asesor_cliente,
                r.tipo_operacion,
                r.zona_deseada,
                r.presupuesto_max,
                r.inmueble_matcheado,
                r.precio_lista,
                r.fecha_venc_exclusividad
            )
        })
        .collect();
    escribir_csv(
        "matching",
        "id,cliente_buscador,asesor_cliente,tipo_operacion,zona_deseada,presupuesto_max,inmueble_matcheado,precio_lista,fecha_venc_exclusividad",
        &filas,
    )?;

    let filas: Vec<String> = db
        .legal
        .iter()
        .map(|e| {
            format!(
                "{},{},{},{},{},{},{},{},{}",
                e.cod_inmueble,
                e.propietario,
                bf(e.titulo_propiedad),
                bf(e.cedula_rif),
                bf(e.ficha_catastral),
                bf(e.solvencia_municipal),
                e.liberacion_hipoteca,
                e.borrador_contrato,
                e.estatus_notaria
            )
        })
        .collect();
    escribir_csv(
        "legal",
        "cod_inmueble,propietario,titulo_propiedad,cedula_rif,ficha_catastral,solvencia_municipal,liberacion_hipoteca,borrador_contrato,estatus_notaria",
        &filas,
    )?;

    let filas: Vec<String> = db
        .embudo
        .iter()
        .map(|e| {
            format!(
                "{},{},{},{},{},{},{}",
                e.id_asesor,
                e.nombre_asesor,
                e.llamadas_realizadas,
                e.citas_captacion,
                e.visitas_guiadas,
                e.ofertas_recibidas,
                e.cierres_mes
            )
        })
        .collect();
    escribir_csv(
        "embudo",
        "id_asesor,nombre_asesor,llamadas_realizadas,citas_captacion,visitas_guiadas,ofertas_recibidas,cierres_mes",
        &filas,
    )?;

    let filas: Vec<String> = db
        .reportes
        .iter()
        .map(|r| {
            format!(
                "{},{},{},{},{},{},{},{},{}",
                r.cod_inmueble,
                r.propietario,
                r.telefono,
                r.id_asesor,
                r.visitas_agendadas,
                r.ofertas_recibidas,
                r.canales_publicacion,
                r.notas,
                r.estatus_envio
            )
        })
        .collect();
    escribir_csv(
        "reportes",
        "cod_inmueble,propietario,telefono,id_asesor,visitas_agendadas,ofertas_recibidas,canales_publicacion,notas,estatus_envio",
        &filas,
    )?;

    Ok(())
}

pub fn datos_existen() -> bool {
    Path::new(&ruta("asesores")).exists()
}
