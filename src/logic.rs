// ============================================================================
// logic.rs
// Aqui vive, funcion por funcion, cada formula que estaba en el Excel.
// Cada funcion tiene en el comentario la formula original para que puedas
// auditarla contra la hoja fuente.
// ============================================================================

use crate::models::*;
use std::time::{SystemTime, UNIX_EPOCH};

// ---------------------------------------------------------------------------
// Utilidades de fecha (reemplaza HOY() y FECHA() de Excel, sin dependencias)
// ---------------------------------------------------------------------------

/// Dias desde 1970-01-01 hasta hoy (equivalente a la parte entera de HOY()).
pub fn dias_desde_epoch_hoy() -> i64 {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    (secs / 86400) as i64
}

/// Algoritmo de Howard Hinnant: convierte (y,m,d) -> dias desde 1970-01-01.
/// Sirve tanto para HOY() como para FECHA(y,m,d) y para parsear "YYYY-MM-DD".
fn dias_desde_civil(y: i64, m: i64, d: i64) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = (y - era * 400) as i64; // [0, 399]
    let mp = (m + 9) % 12; // [0, 11]
    let doy = (153 * mp + 2) / 5 + d - 1; // [0, 365]
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy; // [0, 146096]
    era * 146097 + doe - 719468
}

/// Parsea "YYYY-MM-DD" -> dias desde epoch. Si el formato es invalido, devuelve 0.
pub fn parsear_fecha(fecha: &str) -> i64 {
    let partes: Vec<&str> = fecha.trim().split('-').collect();
    if partes.len() != 3 {
        return 0;
    }
    let y = partes[0].parse::<i64>().unwrap_or(1970);
    let m = partes[1].parse::<i64>().unwrap_or(1);
    let d = partes[2].parse::<i64>().unwrap_or(1);
    dias_desde_civil(y, m, d)
}

/// Dias entre HOY() y una fecha objetivo, igual que "=FECHA(y,m,d) - HOY()".
pub fn dias_hasta(fecha_objetivo: &str) -> i64 {
    parsear_fecha(fecha_objetivo) - dias_desde_epoch_hoy()
}

/// Dias transcurridos desde una fecha, igual que "=HOY() - D4".
pub fn dias_desde(fecha: &str) -> i64 {
    dias_desde_epoch_hoy() - parsear_fecha(fecha)
}

// ---------------------------------------------------------------------------
// 1. BD_Asesores  (Dashboard hoja 1 - Ranking Top Producers)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct AsesorCalculado {
    pub id: String,
    pub nombre: String,
    pub ventas_concretadas: i64,
    pub alquileres_concretados: i64,
    pub total_facturado: f64,
    pub pct_asistencia: f64,
    pub score: f64,
    pub posicion_ranking: usize, // 1 = mejor
}

/// Columna D: =CONTAR.SI.CONJUNTO(Cierres.captador=id, tipo="Venta")
///          + =CONTAR.SI.CONJUNTO(Cierres.cerrador=id, tipo="Venta")
/// (si el mismo asesor aparece como captador Y cerrador en el mismo cierre,
/// la hoja original lo cuenta dos veces - se respeta ese comportamiento)
fn contar_operaciones(cierres: &[Cierre], asesor_id: &str, tipo: &str) -> i64 {
    let como_captador = cierres
        .iter()
        .filter(|c| c.id_captador == asesor_id && c.tipo_operacion == tipo)
        .count() as i64;
    let como_cerrador = cierres
        .iter()
        .filter(|c| c.id_cerrador == asesor_id && c.tipo_operacion == tipo)
        .count() as i64;
    como_captador + como_cerrador
}

/// Columna F: =SUMAR.SI(captador=id, pago_asesores) + SUMAR.SI(cerrador=id, pago_asesores)
fn sumar_facturado(cierres: &[Cierre], asesor_id: &str) -> f64 {
    let como_captador: f64 = cierres
        .iter()
        .filter(|c| c.id_captador == asesor_id)
        .map(pago_asesores)
        .sum();
    let como_cerrador: f64 = cierres
        .iter()
        .filter(|c| c.id_cerrador == asesor_id)
        .map(pago_asesores)
        .sum();
    como_captador + como_cerrador
}

/// Recalcula toda la hoja BD_Asesores + el ranking (columna I: JERARQUIA)
/// Score (columna J):
/// = 0.55 * (facturado / max_facturado) * 100
/// + 0.25 * ((ventas*1 + alquileres*0.5) / max(ventas*1+alquileres*0.5)) * 100
/// + 0.10 * (%asistencia) * 100
/// + 0.10 * 80   [placeholder fijo, igual que en la hoja original]
pub fn calcular_asesores(db: &Database) -> Vec<AsesorCalculado> {
    struct Parcial {
        id: String,
        nombre: String,
        ventas: i64,
        alquileres: i64,
        facturado: f64,
        pct_asistencia: f64,
    }

    let parciales: Vec<Parcial> = db
        .asesores
        .iter()
        .map(|a| Parcial {
            id: a.id.clone(),
            nombre: a.nombre.clone(),
            ventas: contar_operaciones(&db.cierres, &a.id, "Venta"),
            alquileres: contar_operaciones(&db.cierres, &a.id, "Alquiler"),
            facturado: sumar_facturado(&db.cierres, &a.id),
            pct_asistencia: a.talleres_asistidos / 10.0, // columna H: =G/10
        })
        .collect();

    let max_facturado = parciales
        .iter()
        .map(|p| p.facturado)
        .fold(0.0_f64, f64::max)
        .max(1e-9); // evita division por cero

    let max_volumen = parciales
        .iter()
        .map(|p| p.ventas as f64 * 1.0 + p.alquileres as f64 * 0.5)
        .fold(0.0_f64, f64::max)
        .max(1e-9);

    let mut resultado: Vec<AsesorCalculado> = parciales
        .iter()
        .map(|p| {
            let volumen = p.ventas as f64 * 1.0 + p.alquileres as f64 * 0.5;
            let score = 0.55 * (p.facturado / max_facturado) * 100.0
                + 0.25 * (volumen / max_volumen) * 100.0
                + 0.10 * p.pct_asistencia * 100.0
                + 0.10 * 80.0;
            AsesorCalculado {
                id: p.id.clone(),
                nombre: p.nombre.clone(),
                ventas_concretadas: p.ventas,
                alquileres_concretados: p.alquileres,
                total_facturado: p.facturado,
                pct_asistencia: p.pct_asistencia,
                score,
                posicion_ranking: 0, // se asigna abajo
            }
        })
        .collect();

    // JERARQUIA(score, rango) sin empates especiales: orden descendente por score
    let mut orden: Vec<usize> = (0..resultado.len()).collect();
    orden.sort_by(|&a, &b| resultado[b].score.partial_cmp(&resultado[a].score).unwrap());
    for (rank, idx) in orden.into_iter().enumerate() {
        resultado[idx].posicion_ranking = rank + 1;
    }
    resultado.sort_by_key(|a| a.posicion_ranking);
    resultado
}

// ---------------------------------------------------------------------------
// 2. Matching_Redes
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NivelMatch {
    Alto,
    Medio,
    NoMatch,
}

impl NivelMatch {
    pub fn etiqueta(&self) -> &'static str {
        match self {
            NivelMatch::Alto => "MATCH ALTO",
            NivelMatch::Medio => "MATCH MEDIO",
            NivelMatch::NoMatch => "NO MATCH",
        }
    }
}

/// La hoja original tenia la formula de nivel de match escrita distinto en
/// cada fila (I4:I6 solo evaluaban ALTO/NO-MATCH, I7 evaluaba ALTO/MEDIO).
/// Aqui se unifica en una sola regla de negocio consistente con 3 niveles,
/// que es lo que las columnas K (Nivel Match) del original insinuaban:
///   precio <= presupuesto              -> MATCH ALTO
///   precio <= presupuesto * 1.10 (10%) -> MATCH MEDIO
///   caso contrario                     -> NO MATCH
pub fn calcular_nivel_match(precio_lista: f64, presupuesto_max: f64) -> NivelMatch {
    if precio_lista <= presupuesto_max {
        NivelMatch::Alto
    } else if precio_lista <= presupuesto_max * 1.10 {
        NivelMatch::Medio
    } else {
        NivelMatch::NoMatch
    }
}

// ---------------------------------------------------------------------------
// 3. Checklist_Legal
// ---------------------------------------------------------------------------

/// =SI(Y(titulo="SI", cedula="SI", catastro="SI", solvencia="SI",
///        borrador_contrato="APROBADO"), "LISTO PARA FIRMA", "INCOMPLETO")
pub fn calcular_estatus_legal(exp: &ExpedienteLegal) -> &'static str {
    if exp.titulo_propiedad
        && exp.cedula_rif
        && exp.ficha_catastral
        && exp.solvencia_municipal
        && exp.borrador_contrato == "APROBADO"
    {
        "LISTO PARA FIRMA"
    } else {
        "INCOMPLETO"
    }
}

// ---------------------------------------------------------------------------
// 4. Embudo_Operativo
// ---------------------------------------------------------------------------

/// =SI.ERROR(cierres/visitas, 0)
pub fn tasa_conversion(e: &EmbudoAsesor) -> f64 {
    if e.visitas_guiadas == 0.0 {
        0.0
    } else {
        e.cierres_mes / e.visitas_guiadas
    }
}

/// =SI(llamadas>=40, "ALTA PRODUCTIVIDAD", "REQUIERE SEGUIMIENTO")
pub fn nivel_actividad(e: &EmbudoAsesor) -> &'static str {
    if e.llamadas_realizadas >= 40.0 {
        "ALTA PRODUCTIVIDAD"
    } else {
        "REQUIERE SEGUIMIENTO"
    }
}

// ---------------------------------------------------------------------------
// 5. BD_Cierres (comisiones)
// ---------------------------------------------------------------------------

/// =Monto * %Comision * 0.50   (mitad para oficina)
pub fn comision_oficina(c: &Cierre) -> f64 {
    c.monto_operacion * c.pct_comision_total * 0.50
}

/// =Monto * %Comision * 0.50   (mitad para asesores)
pub fn pago_asesores(c: &Cierre) -> f64 {
    c.monto_operacion * c.pct_comision_total * 0.50
}

// ---------------------------------------------------------------------------
// 6. BD_Finanzas
// ---------------------------------------------------------------------------

/// =SUMAR.SI(estatus="Pagado", monto)
pub fn total_pagado(fin: &[TransaccionFinanciera]) -> f64 {
    fin.iter()
        .filter(|t| t.estatus_pago == "Pagado")
        .map(|t| t.monto)
        .sum()
}

/// =SUMAR.SI(estatus="Pendiente", monto)
pub fn total_pendiente(fin: &[TransaccionFinanciera]) -> f64 {
    fin.iter()
        .filter(|t| t.estatus_pago == "Pendiente")
        .map(|t| t.monto)
        .sum()
}

// ---------------------------------------------------------------------------
// 7. BD_Captaciones
// ---------------------------------------------------------------------------

pub fn contar_por_estatus(cap: &[Captacion], estatus: &str) -> i64 {
    cap.iter().filter(|c| c.estatus == estatus).count() as i64
}

// ---------------------------------------------------------------------------
// 8. Dashboard (KPIs)
// ---------------------------------------------------------------------------

pub struct KpisDashboard {
    pub total_comisiones_oficina: f64,
    pub matches_disponibles: i64, // ALTO + MEDIO
    pub expedientes_listos_firma: i64,
    pub pagos_pendientes: f64,
}

pub fn calcular_dashboard(db: &Database) -> KpisDashboard {
    let total_comisiones_oficina: f64 = db.cierres.iter().map(comision_oficina).sum();

    let matches_disponibles = db
        .matching
        .iter()
        .filter(|r| {
            matches!(
                calcular_nivel_match(r.precio_lista, r.presupuesto_max),
                NivelMatch::Alto | NivelMatch::Medio
            )
        })
        .count() as i64;

    let expedientes_listos_firma = db
        .legal
        .iter()
        .filter(|e| calcular_estatus_legal(e) == "LISTO PARA FIRMA")
        .count() as i64;

    let pagos_pendientes = total_pendiente(&db.finanzas);

    KpisDashboard {
        total_comisiones_oficina,
        matches_disponibles,
        expedientes_listos_firma,
        pagos_pendientes,
    }
}
