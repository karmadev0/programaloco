// ============================================================================
// logic.rs
// Aqui vive, funcion por funcion, cada formula del Excel de KAVELA
// (sistema_Kavela_1_1.xlsx). Cada bloque indica la hoja/columna original.
// ============================================================================

use crate::models::*;
use std::time::{SystemTime, UNIX_EPOCH};

// ---------------------------------------------------------------------------
// Parametros y listas (hoja "Configuracion" + constantes de las formulas)
// ---------------------------------------------------------------------------

pub const CUOTA_MENSUAL: f64 = 10.0; // Mensualidad: "Cuota: $10"
pub const DIA_LIMITE: u32 = 5; // "Limite: Dia 5 de cada mes"
pub const DIA_MORA_SEVERA: u32 = 15; // <=5 a tiempo | <=15 con mora | >15 mora severa

/// Periodo del sistema: Abr 2026 – Sep 2027 (18 meses), igual que el Excel.
pub const PERIODO_INICIO: (i32, i32) = (2026, 4);
pub const PERIODO_MESES: i32 = 18;

pub const MESES_ES: [&str; 12] =
    ["Ene", "Feb", "Mar", "Abr", "May", "Jun", "Jul", "Ago", "Sep", "Oct", "Nov", "Dic"];

pub const TIPOS_NEGOCIO: [&str; 4] = ["Venta", "Alquiler", "Opción de Compra", "Administración"];
pub const TIPOS_INMUEBLE: [&str; 9] = [
    "Apartamento", "Casa", "Local Comercial", "Parcela", "Finca", "Galpón", "Oficina", "Suite", "Otro",
];
pub const ETAPAS_CRM: [&str; 8] =
    ["Nuevo", "Contactado", "Calificado", "Visita", "Oferta", "Negociación", "Cerrado", "Perdido"];
pub const ESTADOS_INV: [&str; 6] =
    ["Disponible", "Reservado", "Negociando", "Vendido", "Alquilado", "Retirado"];
pub const FINES: [&str; 5] = ["Residencial", "Comercial", "Agro", "Industrial", "Mixto"];
pub const FUENTES_CRM: [&str; 6] =
    ["Referido", "Redes Sociales", "Portal Inmobiliario", "Walk-in", "WhatsApp", "Otro"];
pub const DIAS_GUARDIA: [&str; 5] = ["Lunes", "Martes", "Miércoles", "Jueves", "Viernes"];
pub const CARGOS: [&str; 3] = ["Broker/Abg", "Agente Inmobiliario", "Administrativo"];

pub fn lista(v: &[&str]) -> Vec<String> {
    v.iter().map(|s| s.to_string()).collect()
}

// ---------------------------------------------------------------------------
// Utilidades de fecha (reemplaza HOY(), FECHA(), TEXTO(), EOMONTH() sin dependencias)
// ---------------------------------------------------------------------------

pub fn dias_desde_epoch_hoy() -> i64 {
    let secs = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();
    (secs / 86400) as i64
}

/// Algoritmo de Howard Hinnant: (y,m,d) -> dias desde 1970-01-01.
fn dias_desde_civil(y: i64, m: i64, d: i64) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = y - era * 400;
    let mp = (m + 9) % 12;
    let doy = (153 * mp + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146097 + doe - 719468
}

/// Inversa: dias desde 1970-01-01 -> (anio, mes, dia).
pub fn civil_desde_dias(z: i64) -> (i64, i64, i64) {
    let z = z + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    (y, m, d)
}

pub fn hoy_civil() -> (i64, i64, i64) {
    civil_desde_dias(dias_desde_epoch_hoy())
}

/// "AAAA-MM-DD" de hoy.
pub fn hoy_iso() -> String {
    let (y, m, d) = hoy_civil();
    format!("{:04}-{:02}-{:02}", y, m, d)
}

/// "AAAA-MM" del mes actual.
pub fn mes_actual() -> String {
    let (y, m, _) = hoy_civil();
    format!("{:04}-{:02}", y, m)
}

fn dias_del_mes(y: i64, m: i64) -> i64 {
    match m {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        _ => {
            if (y % 4 == 0 && y % 100 != 0) || y % 400 == 0 { 29 } else { 28 }
        }
    }
}

/// Acepta "AAAA-MM-DD", "DD/MM/AAAA" o "DD-MM-AAAA"; valida que la fecha exista
/// y devuelve siempre "AAAA-MM-DD".
pub fn normalizar_fecha(s: &str) -> Option<String> {
    let t = s.trim();
    let sep = if t.contains('/') { '/' } else { '-' };
    let p: Vec<&str> = t.split(sep).collect();
    if p.len() != 3 {
        return None;
    }
    let n: Vec<i64> = p.iter().filter_map(|x| x.trim().parse::<i64>().ok()).collect();
    if n.len() != 3 {
        return None;
    }
    let (y, m, d) = if p[0].trim().len() == 4 { (n[0], n[1], n[2]) } else { (n[2], n[1], n[0]) };
    if !(1900..=2200).contains(&y) || !(1..=12).contains(&m) || d < 1 || d > dias_del_mes(y, m) {
        return None;
    }
    Some(format!("{:04}-{:02}-{:02}", y, m, d))
}

/// "2026-04-17" -> dias desde epoch (None si es invalida).
pub fn dias_de_fecha(fecha: &str) -> Option<i64> {
    let f = normalizar_fecha(fecha)?;
    let p: Vec<i64> = f.split('-').filter_map(|x| x.parse().ok()).collect();
    Some(dias_desde_civil(p[0], p[1], p[2]))
}

/// "2026-04-17" -> "17/04/2026" (formato del Excel: DD/MM/YYYY).
pub fn fecha_dmy(fecha: &str) -> String {
    match normalizar_fecha(fecha) {
        Some(f) => {
            let p: Vec<&str> = f.split('-').collect();
            format!("{}/{}/{}", p[2], p[1], p[0])
        }
        None => fecha.to_string(),
    }
}

/// Mes de una fecha: "2026-04-17" -> "2026-04" ("" si es invalida). Equivale a R/S del Excel.
pub fn mes_de(fecha: &str) -> String {
    match normalizar_fecha(fecha) {
        Some(f) => f[..7].to_string(),
        None => String::new(),
    }
}

/// "2026-04" -> "Abr 2026".
pub fn etiqueta_mes(mes: &str) -> String {
    let p: Vec<&str> = mes.split('-').collect();
    if p.len() == 2 {
        if let (Ok(y), Ok(m)) = (p[0].parse::<i32>(), p[1].parse::<usize>()) {
            if (1..=12).contains(&m) {
                return format!("{} {}", MESES_ES[m - 1], y);
            }
        }
    }
    mes.to_string()
}

/// Los 18 meses del sistema: ["2026-04", ..., "2027-09"].
pub fn periodo() -> Vec<String> {
    let (mut y, mut m) = PERIODO_INICIO;
    let mut v = Vec::new();
    for _ in 0..PERIODO_MESES {
        v.push(format!("{:04}-{:02}", y, m));
        m += 1;
        if m > 12 {
            m = 1;
            y += 1;
        }
    }
    v
}

/// Nombre del dia de la semana para una fecha (1970-01-01 fue jueves).
pub fn dia_semana(fecha: &str) -> Option<&'static str> {
    let d = dias_de_fecha(fecha)?;
    let nombres = ["Domingo", "Lunes", "Martes", "Miércoles", "Jueves", "Viernes", "Sábado"];
    Some(nombres[((d + 4).rem_euclid(7)) as usize])
}

/// Minusculas + sin tildes: para comparar nombres sin importar mayusculas/acentos.
pub fn normalizar(s: &str) -> String {
    s.to_lowercase()
        .chars()
        .map(|c| match c {
            'á' | 'à' | 'ä' => 'a',
            'é' | 'è' | 'ë' => 'e',
            'í' | 'ì' | 'ï' => 'i',
            'ó' | 'ò' | 'ö' => 'o',
            'ú' | 'ù' | 'ü' => 'u',
            'ñ' => 'n',
            c => c,
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Registro_Negocios (columnas calculadas P, Q, R, S)
// ---------------------------------------------------------------------------

/// P: =IFERROR(O*D,0)   Comision del asesor = comision total x formato %
pub fn comision_asesor(n: &Negocio) -> f64 {
    n.comision_total * n.formato
}

/// Q: =IFERROR(O-P,0)   Comision de la oficina (Kavela) = lo que resta
pub fn comision_oficina(n: &Negocio) -> f64 {
    n.comision_total - comision_asesor(n)
}

// ---------------------------------------------------------------------------
// Asesores / Dashboard (desempeno acumulado por asesor)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct Desempeno {
    pub codigo: String,
    pub nombre: String,   // nombre_corto
    pub negocios: usize,  // COUNTIF(Registro!C, nombre)
    pub ventas: usize,    // COUNTIFS(... "Venta")
    pub alquileres: usize, // COUNTIFS(... "Alquiler")
    pub volumen: f64,     // SUMIF(... I)
    pub com_asesor: f64,  // SUMIF(... P)
    pub com_oficina: f64, // SUMIF(... Q)
    pub captaciones: usize, // COUNTIF(Registro!E, "*nombre*")
    pub puntaje: f64,
    pub ranking: usize,
    pub estado: String,
    pub ultimo_negocio: String,
}

/// Captaciones: cuenta los negocios cuya "Punta Captadora" contiene el nombre del
/// asesor (busqueda parcial sin importar mayusculas/tildes, como el "*nombre*" del Excel).
pub fn es_captacion_de(n: &Negocio, nombre_corto: &str) -> bool {
    let nombre = normalizar(nombre_corto.trim());
    !nombre.is_empty() && normalizar(&n.punta_captadora).contains(&nombre)
}

/// Dashboard J: =Volumen*0.5 + Negocios*200 + ComAsesor*0.3 + Captaciones*100
pub fn puntaje(volumen: f64, negocios: usize, com_asesor: f64, captaciones: usize) -> f64 {
    volumen * 0.5 + negocios as f64 * 200.0 + com_asesor * 0.3 + captaciones as f64 * 100.0
}

pub fn calcular_desempeno(db: &Database) -> Vec<Desempeno> {
    let mut v: Vec<Desempeno> = db
        .asesores
        .iter()
        .map(|a| {
            let propios: Vec<&Negocio> =
                db.negocios.iter().filter(|n| normalizar(&n.asesor) == normalizar(&a.nombre_corto)).collect();
            let volumen: f64 = propios.iter().map(|n| n.monto).sum();
            let com_asesor: f64 = propios.iter().map(|n| comision_asesor(n)).sum();
            let com_oficina: f64 = propios.iter().map(|n| comision_oficina(n)).sum();
            let captaciones = db.negocios.iter().filter(|n| es_captacion_de(n, &a.nombre_corto)).count();
            let ultimo = propios
                .iter()
                .filter_map(|n| normalizar_fecha(&n.fecha))
                .max()
                .map(|f| fecha_dmy(&f))
                .unwrap_or_else(|| "Sin negocios".to_string());
            // Asesores!G: Broker/Abg y Administrativo siempre "Activo"; los agentes
            // quedan "Sin negocios" mientras no tengan ninguno registrado.
            let estado = if a.cargo == "Broker/Abg" || a.cargo == "Administrativo" || !propios.is_empty() {
                "Activo"
            } else {
                "Sin negocios"
            };
            Desempeno {
                codigo: a.codigo.clone(),
                nombre: a.nombre_corto.clone(),
                negocios: propios.len(),
                ventas: propios.iter().filter(|n| n.tipo_negocio == "Venta").count(),
                alquileres: propios.iter().filter(|n| n.tipo_negocio == "Alquiler").count(),
                volumen,
                com_asesor,
                com_oficina,
                captaciones,
                puntaje: puntaje(volumen, propios.len(), com_asesor, captaciones),
                ranking: 0,
                estado: estado.to_string(),
                ultimo_negocio: ultimo,
            }
        })
        .collect();
    // K: =RANK(puntaje, rango, 0)  -> empates comparten posicion
    let puntajes: Vec<f64> = v.iter().map(|d| d.puntaje).collect();
    for d in v.iter_mut() {
        d.ranking = puntajes.iter().filter(|&&p| p > d.puntaje).count() + 1;
    }
    v
}

#[derive(Debug, Clone)]
pub struct KpisDashboard {
    pub total_negocios: usize,
    pub ventas: usize,
    pub alquileres: usize,
    pub volumen: f64,
    pub comision_total: f64,   // "COMIS. TOTAL"
    pub comision_oficina: f64, // "COM. KAVELA"
    pub top_producer: String,  // mayor volumen
    pub top_cierres: String,   // mas negocios
    pub top_captador: String,  // mas captaciones
    pub top_comision: String,  // mayor comision de asesor
}

/// Nombre del asesor con el valor maximo (si el maximo es 0 -> "-").
fn top_por<F: Fn(&Desempeno) -> f64>(d: &[Desempeno], f: F) -> String {
    let mut mejor: Option<(&Desempeno, f64)> = None;
    for x in d {
        let val = f(x);
        if val > 0.0 && mejor.map(|(_, m)| val > m).unwrap_or(true) {
            mejor = Some((x, val));
        }
    }
    mejor.map(|(x, _)| x.nombre.clone()).unwrap_or_else(|| "-".to_string())
}

pub fn calcular_dashboard(db: &Database) -> KpisDashboard {
    let d = calcular_desempeno(db);
    KpisDashboard {
        total_negocios: db.negocios.len(),
        ventas: db.negocios.iter().filter(|n| n.tipo_negocio == "Venta").count(),
        alquileres: db.negocios.iter().filter(|n| n.tipo_negocio == "Alquiler").count(),
        volumen: db.negocios.iter().map(|n| n.monto).sum(),
        comision_total: db.negocios.iter().map(|n| n.comision_total).sum(),
        comision_oficina: db.negocios.iter().map(comision_oficina).sum(),
        top_producer: top_por(&d, |x| x.volumen),
        top_cierres: top_por(&d, |x| x.negocios as f64),
        top_captador: top_por(&d, |x| x.captaciones as f64),
        top_comision: top_por(&d, |x| x.com_asesor),
    }
}

// ---------------------------------------------------------------------------
// Rankings (ranking mensual) y TOP_Mensual
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct RankingMes {
    pub mes: String, // "2026-04"
    pub asesor: String,
    pub volumen: f64,
    pub negocios: usize,
    pub com_asesor: f64,
    pub captaciones: usize,
    pub pos_volumen: usize,
    pub pos_negocios: usize,
    pub pos_comision: usize,
}

/// Rankings!A:I para los 18 meses x todos los asesores. Pos = COUNTIFS(mes, valor > este)+1
pub fn ranking_mensual(db: &Database) -> Vec<RankingMes> {
    let mut todo = Vec::new();
    for mes in periodo() {
        let mut filas: Vec<RankingMes> = db
            .asesores
            .iter()
            .map(|a| {
                let propios: Vec<&Negocio> = db
                    .negocios
                    .iter()
                    .filter(|n| mes_de(&n.fecha) == mes && normalizar(&n.asesor) == normalizar(&a.nombre_corto))
                    .collect();
                RankingMes {
                    mes: mes.clone(),
                    asesor: a.nombre_corto.clone(),
                    volumen: propios.iter().map(|n| n.monto).sum(),
                    negocios: propios.len(),
                    com_asesor: propios.iter().map(|n| comision_asesor(n)).sum(),
                    captaciones: db
                        .negocios
                        .iter()
                        .filter(|n| mes_de(&n.fecha) == mes && es_captacion_de(n, &a.nombre_corto))
                        .count(),
                    pos_volumen: 0,
                    pos_negocios: 0,
                    pos_comision: 0,
                }
            })
            .collect();
        let vol: Vec<f64> = filas.iter().map(|f| f.volumen).collect();
        let neg: Vec<usize> = filas.iter().map(|f| f.negocios).collect();
        let com: Vec<f64> = filas.iter().map(|f| f.com_asesor).collect();
        for f in filas.iter_mut() {
            f.pos_volumen = vol.iter().filter(|&&x| x > f.volumen).count() + 1;
            f.pos_negocios = neg.iter().filter(|&&x| x > f.negocios).count() + 1;
            f.pos_comision = com.iter().filter(|&&x| x > f.com_asesor).count() + 1;
        }
        todo.extend(filas);
    }
    todo
}

#[derive(Debug, Clone)]
pub struct TopMes {
    pub mes: String,
    pub top_producer: String,
    pub volumen: f64,
    pub top_cierres: String,
    pub n_cierres: usize,
    pub top_captador: String,
    pub captaciones: usize,
    pub top_comision: String,
    pub comision: f64,
    pub segundo: String,
    pub vol2: f64,
    pub tercero: String,
    pub vol3: f64,
    pub negocios_mes: usize,
    pub com_oficina: f64,
}

/// TOP_Mensual. Si en el mes nadie tiene valor (>0) se muestra "-" (en el Excel salia el
/// primer asesor de la lista por un empate en cero).
pub fn top_mensual(db: &Database) -> Vec<TopMes> {
    let rk = ranking_mensual(db);
    periodo()
        .into_iter()
        .map(|mes| {
            let filas: Vec<&RankingMes> = rk.iter().filter(|r| r.mes == mes).collect();
            let mejor = |f: &dyn Fn(&RankingMes) -> f64| -> (String, f64) {
                let mut b: Option<(&RankingMes, f64)> = None;
                for r in &filas {
                    let v = f(r);
                    if v > 0.0 && b.map(|(_, m)| v > m).unwrap_or(true) {
                        b = Some((r, v));
                    }
                }
                b.map(|(r, v)| (r.asesor.clone(), v)).unwrap_or(("-".to_string(), 0.0))
            };
            let (tp, tpv) = mejor(&|r| r.volumen);
            let (tc, tcv) = mejor(&|r| r.negocios as f64);
            let (tk, tkv) = mejor(&|r| r.captaciones as f64);
            let (tm, tmv) = mejor(&|r| r.com_asesor);
            let mut orden: Vec<&RankingMes> = filas.iter().copied().filter(|r| r.volumen > 0.0).collect();
            orden.sort_by(|a, b| b.volumen.partial_cmp(&a.volumen).unwrap());
            let (s, sv) = orden.get(1).map(|r| (r.asesor.clone(), r.volumen)).unwrap_or(("-".into(), 0.0));
            let (t, tv) = orden.get(2).map(|r| (r.asesor.clone(), r.volumen)).unwrap_or(("-".into(), 0.0));
            let del_mes: Vec<&Negocio> = db.negocios.iter().filter(|n| mes_de(&n.fecha) == mes).collect();
            TopMes {
                mes,
                top_producer: tp,
                volumen: tpv,
                top_cierres: tc,
                n_cierres: tcv as usize,
                top_captador: tk,
                captaciones: tkv as usize,
                top_comision: tm,
                comision: tmv,
                segundo: s,
                vol2: sv,
                tercero: t,
                vol3: tv,
                negocios_mes: del_mes.len(),
                com_oficina: del_mes.iter().map(|n| comision_oficina(n)).sum(),
            }
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Mensualidad
// ---------------------------------------------------------------------------

/// =SI(vacio,"Pendiente",SI(dia<=5,"Pagado a Tiempo",SI(dia<=15,"Pagado con Mora","Mora Severa")))
pub fn estatus_mensualidad(dia: Option<u32>) -> &'static str {
    match dia {
        None => "Pendiente",
        Some(d) if d <= DIA_LIMITE => "Pagado a Tiempo",
        Some(d) if d <= DIA_MORA_SEVERA => "Pagado con Mora",
        Some(_) => "Mora Severa",
    }
}

/// Pagan mensualidad los agentes (Broker/Abg y Administrativo no, como en el Excel).
pub fn asesores_que_pagan(db: &Database) -> Vec<&Asesor> {
    db.asesores.iter().filter(|a| a.cargo == "Agente Inmobiliario").collect()
}

pub fn dia_pago(db: &Database, codigo: &str, mes: &str) -> Option<u32> {
    db.mensualidad.iter().find(|p| p.codigo == codigo && p.mes == mes && p.dia > 0).map(|p| p.dia)
}

/// Fila de la grilla: una por (asesor que paga x mes del periodo hasta el mes actual).
/// Orden: mes mas reciente primero, luego codigo de asesor.
pub fn celdas_mensualidad(db: &Database, hasta_mes: &str) -> Vec<(String, String)> {
    let mut meses: Vec<String> = periodo().into_iter().filter(|m| m.as_str() <= hasta_mes).collect();
    meses.reverse();
    let mut v = Vec::new();
    for m in meses {
        for a in asesores_que_pagan(db) {
            v.push((a.codigo.clone(), m.clone()));
        }
    }
    v
}

/// Establece (o borra, con None) el pago de un asesor en un mes.
pub fn fijar_pago(db: &mut Database, codigo: &str, mes: &str, dia: Option<u32>) {
    db.mensualidad.retain(|p| !(p.codigo == codigo && p.mes == mes));
    if let Some(d) = dia {
        db.mensualidad.push(PagoMensualidad { codigo: codigo.to_string(), mes: mes.to_string(), dia: d });
    }
}

// ---------------------------------------------------------------------------
// Finanzas_Gastos
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct FinanzasMes {
    pub mes: String,
    pub ingr_comisiones: f64, // SUMIFS(Registro!O, fecha en el mes)  (comision TOTAL)
    pub ingr_mensualidad: f64,
    pub total_ingresos: f64,
    pub arriendo: f64,
    pub servicios: f64,
    pub marketing: f64,
    pub nomina: f64,
    pub total_egresos: f64,
    pub utilidad: f64,
    pub margen: f64,
}

/// C: =COUNTIF("Pagado a Tiempo")*10 + COUNTIF("Pagado con Mora")*10
/// (igual que el Excel, "Mora Severa" no suma como ingreso)
pub fn ingreso_mensualidad(db: &Database, mes: &str) -> f64 {
    let n = asesores_que_pagan(db)
        .iter()
        .filter(|a| {
            matches!(
                estatus_mensualidad(dia_pago(db, &a.codigo, mes)),
                "Pagado a Tiempo" | "Pagado con Mora"
            )
        })
        .count();
    n as f64 * CUOTA_MENSUAL
}

pub fn finanzas_mensuales(db: &Database) -> Vec<FinanzasMes> {
    periodo()
        .into_iter()
        .map(|mes| {
            let ingr_comisiones: f64 =
                db.negocios.iter().filter(|n| mes_de(&n.fecha) == mes).map(|n| n.comision_total).sum();
            let ingr_mensualidad = ingreso_mensualidad(db, &mes);
            let e = db.egresos.iter().find(|e| e.mes == mes);
            let (arriendo, servicios, marketing, nomina) =
                e.map(|e| (e.arriendo, e.servicios, e.marketing, e.nomina)).unwrap_or((0.0, 0.0, 0.0, 0.0));
            let total_ingresos = ingr_comisiones + ingr_mensualidad;
            let total_egresos = arriendo + servicios + marketing + nomina;
            let utilidad = total_ingresos - total_egresos;
            FinanzasMes {
                mes,
                ingr_comisiones,
                ingr_mensualidad,
                total_ingresos,
                arriendo,
                servicios,
                marketing,
                nomina,
                total_egresos,
                utilidad,
                margen: if total_ingresos != 0.0 { utilidad / total_ingresos } else { 0.0 },
            }
        })
        .collect()
}

// ---------------------------------------------------------------------------
// CRM_Clientes
// ---------------------------------------------------------------------------

/// M: probabilidad de cierre segun la etapa del pipeline.
pub fn prob_cierre(etapa: &str) -> Option<f64> {
    match etapa {
        "Nuevo" => Some(0.1),
        "Contactado" => Some(0.2),
        "Calificado" => Some(0.4),
        "Visita" => Some(0.6),
        "Oferta" => Some(0.8),
        "Negociación" => Some(0.9),
        "Cerrado" => Some(1.0),
        "Perdido" => Some(0.0),
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// Guardia_Juridica (buscador por fecha)
// ---------------------------------------------------------------------------

/// B12/B13: dia de la semana de la fecha y abogado asignado a ese dia.
pub fn abogado_de_guardia(db: &Database, fecha: &str) -> String {
    let dia = match dia_semana(fecha) {
        Some(d) => d,
        None => return "Fecha no válida (use AAAA-MM-DD o DD/MM/AAAA)".to_string(),
    };
    match db.guardia.iter().find(|g| normalizar(&g.dia) == normalizar(dia)) {
        Some(g) => format!("{}: {}", dia, g.abogado),
        None => format!("{}: Fin de semana / no laborable", dia),
    }
}
