// ============================================================================
// modulos.rs
// Capa comun para las DOS interfaces (Slint y Tauri): a partir de la base de
// datos arma lo que se muestra (tablas), los formularios, y valida/guarda.
// Asi cada interfaz solo dibuja; las reglas de KAVELA viven en un solo lugar.
// ============================================================================

use crate::logic::*;
use crate::models::*;

pub const M_DASHBOARD: i32 = 0;
pub const M_NEGOCIOS: i32 = 1;
pub const M_ASESORES: i32 = 2;
pub const M_MENSUALIDAD: i32 = 3;
pub const M_FINANZAS: i32 = 4;
pub const M_INVENTARIO: i32 = 5;
pub const M_CRM: i32 = 6;
pub const M_GUARDIA: i32 = 7;
pub const M_TOP: i32 = 8;
pub const M_RANKING: i32 = 9;
pub const M_CONFIG: i32 = 10;
pub const M_GRAFICOS: i32 = 11;

/// Lo que se dibuja en un listado.
#[derive(Debug, Clone, Default)]
pub struct Vista {
    pub titulo: String,
    pub ayuda: String,
    pub headers: Vec<String>,
    /// Cuantas columnas caben en la fila; el resto solo aparece en el detalle de la fila.
    pub visibles: usize,
    pub filas: Vec<Vec<String>>,
    pub puede_crear: bool,
    pub puede_editar: bool,
    pub puede_eliminar: bool,
}

/// Un campo de formulario. `opciones` vacio = texto libre; con opciones = lista desplegable.
#[derive(Debug, Clone)]
pub struct CampoForm {
    pub etiqueta: String,
    pub valor: String,
    pub opciones: Vec<String>,
}

fn campo(etiqueta: &str, valor: impl Into<String>) -> CampoForm {
    CampoForm { etiqueta: etiqueta.to_string(), valor: valor.into(), opciones: vec![] }
}

fn selector(etiqueta: &str, valor: &str, opciones: Vec<String>) -> CampoForm {
    let v = if valor.is_empty() { opciones.first().cloned().unwrap_or_default() } else { valor.to_string() };
    CampoForm { etiqueta: etiqueta.to_string(), valor: v, opciones }
}

// ---------------------------------------------------------------------------
// Formato de salida
// ---------------------------------------------------------------------------

pub fn dinero(v: f64) -> String {
    let s = format!("{:.2}", v.abs());
    let (ent, dec) = s.split_once('.').unwrap_or((&s, "00"));
    let mut g = String::new();
    for (i, c) in ent.chars().enumerate() {
        if i > 0 && (ent.len() - i) % 3 == 0 {
            g.push(',');
        }
        g.push(c);
    }
    let neg = v < 0.0 && s != "0.00";
    format!("{}${}.{}", if neg { "-" } else { "" }, g, dec)
}

pub fn porcentaje(v: f64) -> String {
    let p = v * 100.0;
    if (p - p.round()).abs() < 0.05 { format!("{:.0}%", p) } else { format!("{:.1}%", p) }
}

/// Numero para un campo de formulario: 0 -> vacio, entero sin decimales.
fn num_txt(v: f64) -> String {
    if v == 0.0 {
        String::new()
    } else if v == v.trunc() {
        format!("{}", v as i64)
    } else {
        format!("{}", v)
    }
}

fn pct_txt(v: f64) -> String {
    if v == 0.0 { String::new() } else { porcentaje(v) }
}

// ---------------------------------------------------------------------------
// Validacion de campos
// ---------------------------------------------------------------------------

fn num_opt(s: &str, campo: &str) -> Result<f64, String> {
    let t = s.trim().replace(',', ".");
    if t.is_empty() {
        return Ok(0.0);
    }
    t.parse::<f64>().map_err(|_| format!("El campo '{}' debe ser un número válido.", campo))
}

fn num_req(s: &str, campo: &str) -> Result<f64, String> {
    if s.trim().is_empty() {
        return Err(format!("El campo '{}' es obligatorio.", campo));
    }
    num_opt(s, campo)
}

/// Acepta "50", "50%", "0.5", "0,55": devuelve la fraccion (0.5).
fn pct_opt(s: &str, campo: &str) -> Result<f64, String> {
    let t = s.trim().trim_end_matches('%').trim().replace(',', ".");
    if t.is_empty() {
        return Ok(0.0);
    }
    let v: f64 = t.parse().map_err(|_| format!("El campo '{}' debe ser un porcentaje válido.", campo))?;
    let v = if v > 1.0 { v / 100.0 } else { v };
    if !(0.0..=1.0).contains(&v) {
        return Err(format!("El campo '{}' debe estar entre 0% y 100%.", campo));
    }
    Ok(v)
}

fn fecha_req(s: &str, campo: &str) -> Result<String, String> {
    normalizar_fecha(s).ok_or_else(|| format!("'{}' debe ser una fecha válida (AAAA-MM-DD o DD/MM/AAAA).", campo))
}

fn fecha_opt(s: &str, campo: &str) -> Result<String, String> {
    if s.trim().is_empty() { Ok(String::new()) } else { fecha_req(s, campo) }
}

fn en_lista(s: &str, opciones: &[String], campo: &str) -> Result<String, String> {
    opciones
        .iter()
        .find(|o| normalizar(o) == normalizar(s.trim()))
        .cloned()
        .ok_or_else(|| format!("'{}' no es una opción válida de '{}'.", s.trim(), campo))
}

fn texto(s: &str) -> String {
    s.trim().to_string()
}

fn nombres_cortos(db: &Database) -> Vec<String> {
    db.asesores.iter().map(|a| a.nombre_corto.clone()).collect()
}

fn celda(v: &str) -> String {
    v.to_string()
}

// ---------------------------------------------------------------------------
// Orden de las filas (indice de fila mostrada -> indice en la base de datos)
// ---------------------------------------------------------------------------

fn orden(db: &Database, m: i32) -> Vec<usize> {
    match m {
        M_NEGOCIOS => {
            let mut v: Vec<usize> = (0..db.negocios.len()).collect();
            v.sort_by(|&a, &b| {
                (normalizar_fecha(&db.negocios[b].fecha), db.negocios[b].id)
                    .cmp(&(normalizar_fecha(&db.negocios[a].fecha), db.negocios[a].id))
            });
            v
        }
        M_INVENTARIO => {
            let mut v: Vec<usize> = (0..db.inventario.len()).collect();
            v.sort_by(|&a, &b| db.inventario[b].id.cmp(&db.inventario[a].id));
            v
        }
        M_CRM => {
            let mut v: Vec<usize> = (0..db.clientes.len()).collect();
            v.sort_by(|&a, &b| db.clientes[b].id.cmp(&db.clientes[a].id));
            v
        }
        M_ASESORES => (0..db.asesores.len()).collect(),
        M_GUARDIA => (0..db.guardia.len()).collect(),
        _ => vec![],
    }
}

fn fila_db(db: &Database, m: i32, idx: usize) -> Result<usize, String> {
    orden(db, m).get(idx).copied().ok_or_else(|| "La fila seleccionada ya no existe.".to_string())
}

// ---------------------------------------------------------------------------
// Listados
// ---------------------------------------------------------------------------

fn hs(v: &[&str]) -> Vec<String> {
    v.iter().map(|s| s.to_string()).collect()
}

pub fn titulo_modulo(m: i32) -> &'static str {
    match m {
        M_DASHBOARD => "Dashboard",
        M_NEGOCIOS => "Registro de Negocios",
        M_ASESORES => "Asesores",
        M_MENSUALIDAD => "Mensualidad",
        M_FINANZAS => "Finanzas y Gastos",
        M_INVENTARIO => "Inventario",
        M_CRM => "CRM Clientes",
        M_GUARDIA => "Guardia Jurídica",
        M_TOP => "TOP Mensual",
        M_RANKING => "Rankings Mensuales",
        M_CONFIG => "Configuración",
        M_GRAFICOS => "Gráficos",
        _ => "",
    }
}

pub fn vista(db: &Database, m: i32) -> Vista {
    let mut v = Vista { titulo: titulo_modulo(m).to_string(), ..Default::default() };
    match m {
        M_NEGOCIOS => {
            v.ayuda = "ÚNICA hoja de entrada de datos: todo lo demás se calcula solo. Toque una fila para ver todas las columnas.".into();
            v.headers = hs(&[
                "ID", "Fecha", "Asesor", "Tipo", "Monto ($)", "Ubicación", "Comisión Total", "Com. Asesor",
                "Com. Oficina", "Mes", "Formato", "Punta Captadora", "Punta Compradora", "Punta Arrendataria",
                "Asesor Otra Punta", "Tipo Inmueble", "Canon ($)", "Fin", "Origen", "Notas",
            ]);
            v.visibles = 9;
            v.puede_crear = true;
            v.puede_editar = true;
            v.puede_eliminar = true;
            v.filas = orden(db, m)
                .into_iter()
                .map(|i| {
                    let n = &db.negocios[i];
                    vec![
                        n.id.to_string(), fecha_dmy(&n.fecha), celda(&n.asesor), celda(&n.tipo_negocio),
                        dinero(n.monto), celda(&n.ubicacion), dinero(n.comision_total),
                        dinero(comision_asesor(n)), dinero(comision_oficina(n)),
                        etiqueta_mes(&mes_de(&n.fecha)), porcentaje(n.formato), celda(&n.punta_captadora),
                        celda(&n.punta_compradora), celda(&n.punta_arrendataria), celda(&n.asesor_otra_punta),
                        celda(&n.tipo_inmueble), if n.canon == 0.0 { String::new() } else { dinero(n.canon) },
                        celda(&n.fin), celda(&n.origen), celda(&n.notas),
                    ]
                })
                .collect();
        }
        M_ASESORES => {
            v.ayuda = "Directorio del equipo. Negocios, volumen y estado se calculan desde el Registro de Negocios.".into();
            v.headers = hs(&[
                "Código", "Asesor", "Cargo", "Formato", "Estado", "N° Negocios", "Volumen ($)", "Com. Asesor ($)",
                "Último Negocio", "Nombre completo", "Cédula", "Fecha ingreso", "Teléfono",
            ]);
            v.visibles = 9;
            v.puede_crear = true;
            v.puede_editar = true;
            v.puede_eliminar = true;
            let d = calcular_desempeno(db);
            v.filas = db
                .asesores
                .iter()
                .zip(d.iter())
                .map(|(a, x)| {
                    vec![
                        celda(&a.codigo), celda(&a.nombre_corto), celda(&a.cargo), pct_txt(a.formato),
                        celda(&x.estado), x.negocios.to_string(), dinero(x.volumen), dinero(x.com_asesor),
                        celda(&x.ultimo_negocio), celda(&a.nombre_completo), celda(&a.cedula),
                        if a.fecha_ingreso.is_empty() { String::new() } else { fecha_dmy(&a.fecha_ingreso) },
                        celda(&a.telefono),
                    ]
                })
                .collect();
        }
        M_MENSUALIDAD => {
            v.ayuda = format!(
                "Cuota ${:.0} · Límite día {} · hasta el {} con mora · después, mora severa. Use Editar para poner el día de pago.",
                CUOTA_MENSUAL, DIA_LIMITE, DIA_MORA_SEVERA
            );
            v.headers = hs(&["Mes", "Código", "Asesor", "Día de pago", "Estatus"]);
            v.visibles = 5;
            v.puede_editar = true;
            v.filas = celdas_mensualidad(db, &mes_actual())
                .into_iter()
                .map(|(cod, mes)| {
                    let a = db.asesores.iter().find(|a| a.codigo == cod);
                    let dia = dia_pago(db, &cod, &mes);
                    vec![
                        etiqueta_mes(&mes), cod.clone(),
                        a.map(|a| a.nombre_completo.clone()).unwrap_or_default(),
                        dia.map(|d| d.to_string()).unwrap_or_default(),
                        estatus_mensualidad(dia).to_string(),
                    ]
                })
                .collect();
        }
        M_FINANZAS => {
            v.ayuda = "Los ingresos se calculan solos. Use Editar en un mes para cargar los egresos reales.".into();
            v.headers = hs(&[
                "Mes", "Ingr. Comisiones", "Ingr. Mensualidad", "Total Ingresos", "Arrend. Oficina", "Servicios",
                "Marketing", "Nómina Admin", "Total Egresos", "Utilidad Neta", "Margen",
            ]);
            v.visibles = 11;
            v.puede_editar = true;
            let f = finanzas_mensuales(db);
            let mut filas: Vec<Vec<String>> = f
                .iter()
                .map(|x| {
                    vec![
                        etiqueta_mes(&x.mes), dinero(x.ingr_comisiones), dinero(x.ingr_mensualidad),
                        dinero(x.total_ingresos), dinero(x.arriendo), dinero(x.servicios), dinero(x.marketing),
                        dinero(x.nomina), dinero(x.total_egresos), dinero(x.utilidad), porcentaje(x.margen),
                    ]
                })
                .collect();
            let s = |g: &dyn Fn(&FinanzasMes) -> f64| dinero(f.iter().map(g).sum());
            filas.push(vec![
                "TOTALES".into(), s(&|x| x.ingr_comisiones), s(&|x| x.ingr_mensualidad), s(&|x| x.total_ingresos),
                s(&|x| x.arriendo), s(&|x| x.servicios), s(&|x| x.marketing), s(&|x| x.nomina),
                s(&|x| x.total_egresos), s(&|x| x.utilidad), String::new(),
            ]);
            v.filas = filas;
        }
        M_INVENTARIO => {
            v.ayuda = "Propiedades en cartera.".into();
            v.headers = hs(&[
                "ID", "Fecha", "Captador", "Tipo", "Ubicación", "Precio Venta ($)", "Canon ($)", "Operación",
                "Estado", "Propietario", "Teléfono", "Hab.", "Baños", "M²", "Características", "Notas",
            ]);
            v.visibles = 9;
            v.puede_crear = true;
            v.puede_editar = true;
            v.puede_eliminar = true;
            v.filas = orden(db, m)
                .into_iter()
                .map(|i| {
                    let p = &db.inventario[i];
                    vec![
                        p.id.to_string(), fecha_dmy(&p.fecha), celda(&p.asesor), celda(&p.tipo_inmueble),
                        celda(&p.ubicacion),
                        if p.precio_venta == 0.0 { String::new() } else { dinero(p.precio_venta) },
                        if p.canon == 0.0 { String::new() } else { dinero(p.canon) },
                        celda(&p.tipo_operacion), celda(&p.estado), celda(&p.propietario), celda(&p.telefono),
                        celda(&p.habitaciones), celda(&p.banos), celda(&p.m2), celda(&p.caracteristicas),
                        celda(&p.notas),
                    ]
                })
                .collect();
        }
        M_CRM => {
            v.ayuda = "Seguimiento de clientes. El % de probabilidad se calcula según la etapa del pipeline.".into();
            v.headers = hs(&[
                "ID", "Contacto", "Cliente", "Asesor", "Tipo Negocio", "Presupuesto ($)", "Etapa", "% Prob.",
                "Próx. Contacto", "Teléfono", "Email", "Zona", "Tipo Inmueble", "Fuente", "Inmueble Sugerido",
                "Resultado", "Fecha Cierre", "Notas",
            ]);
            v.visibles = 9;
            v.puede_crear = true;
            v.puede_editar = true;
            v.puede_eliminar = true;
            v.filas = orden(db, m)
                .into_iter()
                .map(|i| {
                    let c = &db.clientes[i];
                    vec![
                        c.id.to_string(), fecha_dmy(&c.fecha_contacto), celda(&c.nombre), celda(&c.asesor),
                        celda(&c.tipo_negocio),
                        if c.presupuesto == 0.0 { String::new() } else { dinero(c.presupuesto) },
                        celda(&c.etapa),
                        prob_cierre(&c.etapa).map(porcentaje).unwrap_or_default(),
                        fecha_dmy(&c.proximo_contacto), celda(&c.telefono), celda(&c.email), celda(&c.zona),
                        celda(&c.tipo_inmueble), celda(&c.fuente), celda(&c.inmueble_sugerido),
                        celda(&c.resultado), fecha_dmy(&c.fecha_cierre), celda(&c.notas),
                    ]
                })
                .collect();
        }
        M_GUARDIA => {
            v.ayuda = "Abogado de guardia de lunes a viernes. Use el buscador para ver quién está de guardia en una fecha.".into();
            v.headers = hs(&["Día", "Abogado", "Cédula / Colegio", "Teléfono", "Correo", "Notas"]);
            v.visibles = 6;
            v.puede_editar = true;
            v.filas = db
                .guardia
                .iter()
                .map(|g| {
                    vec![
                        celda(&g.dia), celda(&g.abogado), celda(&g.cedula_colegio), celda(&g.telefono),
                        celda(&g.correo), celda(&g.notas),
                    ]
                })
                .collect();
        }
        M_TOP => {
            v.ayuda = "Cuadro de honor mensual (se calcula solo). \"-\" = nadie tuvo actividad ese mes.".into();
            v.headers = hs(&[
                "Mes", "Top Producer", "Volumen ($)", "Top Cierres", "# Negocios", "Top Captador", "Captaciones",
                "Negocios del Mes", "Com. Kavela ($)", "Top Ventas (Com.)", "Comisión ($)", "2° Volumen",
                "2° ($)", "3° Volumen", "3° ($)",
            ]);
            v.visibles = 9;
            v.filas = top_mensual(db)
                .iter()
                .map(|t| {
                    vec![
                        etiqueta_mes(&t.mes), celda(&t.top_producer), dinero(t.volumen), celda(&t.top_cierres),
                        t.n_cierres.to_string(), celda(&t.top_captador), t.captaciones.to_string(),
                        t.negocios_mes.to_string(), dinero(t.com_oficina), celda(&t.top_comision),
                        dinero(t.comision), celda(&t.segundo), dinero(t.vol2), celda(&t.tercero), dinero(t.vol3),
                    ]
                })
                .collect();
        }
        M_RANKING => {
            v.ayuda = "Ranking mensual por asesor (solo meses/asesores con actividad).".into();
            v.headers = hs(&[
                "Mes", "Asesor", "Volumen ($)", "N° Negocios", "Com. Asesor ($)", "Captaciones", "Pos. Volumen",
                "Pos. Negocios", "Pos. Comisión",
            ]);
            v.visibles = 9;
            let mut r: Vec<RankingMes> = ranking_mensual(db)
                .into_iter()
                .filter(|x| x.volumen > 0.0 || x.negocios > 0 || x.captaciones > 0)
                .collect();
            r.sort_by(|a, b| b.mes.cmp(&a.mes).then(a.pos_volumen.cmp(&b.pos_volumen)));
            v.filas = r
                .iter()
                .map(|x| {
                    vec![
                        etiqueta_mes(&x.mes), celda(&x.asesor), dinero(x.volumen), x.negocios.to_string(),
                        dinero(x.com_asesor), x.captaciones.to_string(), x.pos_volumen.to_string(),
                        x.pos_negocios.to_string(), x.pos_comision.to_string(),
                    ]
                })
                .collect();
        }
        M_CONFIG => {
            v.ayuda = "Listas que usan los formularios. Los asesores se administran en la pestaña Asesores.".into();
            v.headers = hs(&[
                "Asesores", "Tipo Negocio", "Tipo Inmueble", "Etapa CRM", "Estado Inv.", "Fin / Propósito",
                "Fuente CRM", "Guardia",
            ]);
            v.visibles = 8;
            let cols: Vec<Vec<String>> = vec![
                nombres_cortos(db), lista(&TIPOS_NEGOCIO), lista(&TIPOS_INMUEBLE), lista(&ETAPAS_CRM),
                lista(&ESTADOS_INV), lista(&FINES), lista(&FUENTES_CRM), lista(&DIAS_GUARDIA),
            ];
            let alto = cols.iter().map(|c| c.len()).max().unwrap_or(0);
            v.filas = (0..alto)
                .map(|i| cols.iter().map(|c| c.get(i).cloned().unwrap_or_default()).collect())
                .collect();
        }
        _ => {}
    }
    v
}

// ---------------------------------------------------------------------------
// Dashboard y graficos
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Default)]
pub struct Dash {
    pub kpis: Vec<(String, String)>,
    pub honor: Vec<(String, String)>,
    pub headers: Vec<String>,
    pub filas: Vec<Vec<String>>,
}

pub fn dashboard(db: &Database) -> Dash {
    let k = calcular_dashboard(db);
    let d = calcular_desempeno(db);
    let t = |a: &str, b: String| (a.to_string(), b);
    let mut filas: Vec<Vec<String>> = d
        .iter()
        .map(|x| {
            vec![
                x.nombre.clone(), x.codigo.clone(), x.negocios.to_string(), x.ventas.to_string(),
                x.alquileres.to_string(), dinero(x.volumen), dinero(x.com_asesor), dinero(x.com_oficina),
                x.captaciones.to_string(), format!("{:.1}", x.puntaje), x.ranking.to_string(), x.estado.clone(),
            ]
        })
        .collect();
    filas.push(vec![
        "TOTALES".into(), String::new(),
        d.iter().map(|x| x.negocios).sum::<usize>().to_string(),
        d.iter().map(|x| x.ventas).sum::<usize>().to_string(),
        d.iter().map(|x| x.alquileres).sum::<usize>().to_string(),
        dinero(d.iter().map(|x| x.volumen).sum()), dinero(d.iter().map(|x| x.com_asesor).sum()),
        dinero(d.iter().map(|x| x.com_oficina).sum()), d.iter().map(|x| x.captaciones).sum::<usize>().to_string(),
        String::new(), String::new(), String::new(),
    ]);
    Dash {
        kpis: vec![
            t("Total negocios", k.total_negocios.to_string()),
            t("Ventas cerradas", k.ventas.to_string()),
            t("Alquileres", k.alquileres.to_string()),
            t("Volumen total ($)", dinero(k.volumen)),
            t("Comisión total ($)", dinero(k.comision_total)),
            t("Comisión Kavela ($)", dinero(k.comision_oficina)),
        ],
        honor: vec![
            t("Top Producer general", k.top_producer),
            t("Top cierres total", k.top_cierres),
            t("Top captador total", k.top_captador),
            t("Top comisión asesor", k.top_comision),
        ],
        headers: hs(&[
            "Asesor", "Código", "N° Negocios", "Ventas", "Alquileres", "Volumen ($)", "Com. Asesor ($)",
            "Com. Kavela ($)", "Captaciones", "Puntaje", "Ranking", "Estado",
        ]),
        filas,
    }
}

#[derive(Debug, Clone)]
pub struct Grafico {
    pub titulo: String,
    /// (etiqueta, valor, texto a mostrar)
    pub barras: Vec<(String, f64, String)>,
}

pub fn graficos(db: &Database) -> Vec<Grafico> {
    let d = calcular_desempeno(db);
    let mut por_asesor: Vec<&Desempeno> = d.iter().filter(|x| x.volumen > 0.0).collect();
    por_asesor.sort_by(|a, b| b.volumen.partial_cmp(&a.volumen).unwrap());

    let ahora = mes_actual();
    let f = finanzas_mensuales(db);
    let meses: Vec<&FinanzasMes> = f
        .iter()
        .filter(|x| x.mes <= ahora || db.negocios.iter().any(|n| mes_de(&n.fecha) == x.mes))
        .collect();

    vec![
        Grafico {
            titulo: "Volumen por asesor ($)".into(),
            barras: por_asesor.iter().map(|x| (x.nombre.clone(), x.volumen, dinero(x.volumen))).collect(),
        },
        Grafico {
            titulo: "Negocios por mes".into(),
            barras: meses
                .iter()
                .map(|x| {
                    let n = db.negocios.iter().filter(|n| mes_de(&n.fecha) == x.mes).count();
                    (etiqueta_mes(&x.mes), n as f64, n.to_string())
                })
                .collect(),
        },
        Grafico {
            titulo: "Ingresos totales por mes ($)".into(),
            barras: meses.iter().map(|x| (etiqueta_mes(&x.mes), x.total_ingresos, dinero(x.total_ingresos))).collect(),
        },
    ]
}

/// Buscador de la guardia juridica.
pub fn consultar_guardia(db: &Database, fecha: &str) -> String {
    abogado_de_guardia(db, fecha)
}

// ---------------------------------------------------------------------------
// Formularios
// ---------------------------------------------------------------------------

/// Campos del formulario. idx = None -> "Nuevo"; Some(i) -> editar la fila i mostrada.
pub fn campos(db: &Database, m: i32, idx: Option<usize>) -> Result<Vec<CampoForm>, String> {
    let asesores = nombres_cortos(db);
    match m {
        M_NEGOCIOS => {
            let n = match idx {
                Some(i) => Some(&db.negocios[fila_db(db, m, i)?]),
                None => None,
            };
            let g = |f: &dyn Fn(&Negocio) -> String| n.map(f).unwrap_or_default();
            Ok(vec![
                campo("Fecha del negocio (AAAA-MM-DD)", if n.is_some() { g(&|x| x.fecha.clone()) } else { hoy_iso() }),
                selector("Asesor", &g(&|x| x.asesor.clone()), asesores),
                campo("Formato % (vacío = el del asesor)", n.map(|x| pct_txt(x.formato)).unwrap_or_default()),
                campo("Punta captadora", g(&|x| x.punta_captadora.clone())),
                campo("Punta compradora", g(&|x| x.punta_compradora.clone())),
                campo("Punta arrendataria", g(&|x| x.punta_arrendataria.clone())),
                campo("Asesor otra punta", g(&|x| x.asesor_otra_punta.clone())),
                campo("Monto del negocio ($)", n.map(|x| num_txt(x.monto)).unwrap_or_default()),
                selector("Tipo de inmueble", &g(&|x| x.tipo_inmueble.clone()), lista(&TIPOS_INMUEBLE)),
                campo("Ubicación", g(&|x| x.ubicacion.clone())),
                campo("Canon ($, solo alquiler)", n.map(|x| num_txt(x.canon)).unwrap_or_default()),
                selector("Tipo de negocio", &g(&|x| x.tipo_negocio.clone()), lista(&TIPOS_NEGOCIO)),
                selector("Fin / propósito", &g(&|x| x.fin.clone()), lista(&FINES)),
                campo("Comisión total ($)", n.map(|x| num_txt(x.comision_total)).unwrap_or_default()),
                campo("Origen", if n.is_some() { g(&|x| x.origen.clone()) } else { "Nuevo".into() }),
                campo("Notas", g(&|x| x.notas.clone())),
            ])
        }
        M_ASESORES => {
            let a = match idx {
                Some(i) => Some(&db.asesores[fila_db(db, m, i)?]),
                None => None,
            };
            let g = |f: &dyn Fn(&Asesor) -> String| a.map(f).unwrap_or_default();
            Ok(vec![
                campo("Nombre completo", g(&|x| x.nombre_completo.clone())),
                campo("Nombre corto (como se escribe en negocios)", g(&|x| x.nombre_corto.clone())),
                campo("Cédula", g(&|x| x.cedula.clone())),
                selector("Cargo", &g(&|x| x.cargo.clone()), lista(&CARGOS)),
                campo("Formato % (vacío si no aplica)", a.map(|x| pct_txt(x.formato)).unwrap_or_default()),
                campo("Fecha de ingreso (AAAA-MM-DD)", g(&|x| x.fecha_ingreso.clone())),
                campo("Teléfono", g(&|x| x.telefono.clone())),
            ])
        }
        M_MENSUALIDAD => {
            let (cod, mes) = celdas_mensualidad(db, &mes_actual())
                .get(idx.ok_or("La mensualidad solo se edita.")?)
                .cloned()
                .ok_or("La fila seleccionada ya no existe.")?;
            let nombre = db.asesores.iter().find(|a| a.codigo == cod).map(|a| a.nombre_corto.clone()).unwrap_or_default();
            let dia = dia_pago(db, &cod, &mes).map(|d| d.to_string()).unwrap_or_default();
            Ok(vec![campo(&format!("Día de pago de {} en {} (1-31; vacío = pendiente)", nombre, etiqueta_mes(&mes)), dia)])
        }
        M_FINANZAS => {
            let mes = periodo().get(idx.ok_or("Las finanzas solo se editan.")?).cloned().ok_or("La fila TOTALES no se edita.")?;
            let e = db.egresos.iter().find(|e| e.mes == mes);
            let g = |f: &dyn Fn(&EgresoMes) -> f64| e.map(|x| num_txt(f(x))).unwrap_or_default();
            Ok(vec![
                campo(&format!("Arrendamiento oficina ($) — {}", etiqueta_mes(&mes)), g(&|x| x.arriendo)),
                campo("Servicios ($)", g(&|x| x.servicios)),
                campo("Marketing ($)", g(&|x| x.marketing)),
                campo("Nómina administrativa ($)", g(&|x| x.nomina)),
            ])
        }
        M_INVENTARIO => {
            let p = match idx {
                Some(i) => Some(&db.inventario[fila_db(db, m, i)?]),
                None => None,
            };
            let g = |f: &dyn Fn(&Propiedad) -> String| p.map(f).unwrap_or_default();
            Ok(vec![
                campo("Fecha de captación (AAAA-MM-DD)", if p.is_some() { g(&|x| x.fecha.clone()) } else { hoy_iso() }),
                selector("Asesor captador", &g(&|x| x.asesor.clone()), asesores),
                selector("Tipo de inmueble", &g(&|x| x.tipo_inmueble.clone()), lista(&TIPOS_INMUEBLE)),
                campo("Ubicación", g(&|x| x.ubicacion.clone())),
                campo("Precio de venta ($)", p.map(|x| num_txt(x.precio_venta)).unwrap_or_default()),
                campo("Canon de alquiler ($)", p.map(|x| num_txt(x.canon)).unwrap_or_default()),
                selector("Tipo de operación", &g(&|x| x.tipo_operacion.clone()), lista(&TIPOS_NEGOCIO)),
                selector("Estado", &g(&|x| x.estado.clone()), lista(&ESTADOS_INV)),
                campo("Propietario", g(&|x| x.propietario.clone())),
                campo("Teléfono", g(&|x| x.telefono.clone())),
                campo("Habitaciones", g(&|x| x.habitaciones.clone())),
                campo("Baños", g(&|x| x.banos.clone())),
                campo("M² aprox.", g(&|x| x.m2.clone())),
                campo("Características", g(&|x| x.caracteristicas.clone())),
                campo("Notas", g(&|x| x.notas.clone())),
            ])
        }
        M_CRM => {
            let c = match idx {
                Some(i) => Some(&db.clientes[fila_db(db, m, i)?]),
                None => None,
            };
            let g = |f: &dyn Fn(&Cliente) -> String| c.map(f).unwrap_or_default();
            Ok(vec![
                campo("Fecha de contacto (AAAA-MM-DD)", if c.is_some() { g(&|x| x.fecha_contacto.clone()) } else { hoy_iso() }),
                campo("Nombre del cliente", g(&|x| x.nombre.clone())),
                campo("Teléfono", g(&|x| x.telefono.clone())),
                campo("Email", g(&|x| x.email.clone())),
                campo("Zona de interés", g(&|x| x.zona.clone())),
                selector("Tipo de inmueble", &g(&|x| x.tipo_inmueble.clone()), lista(&TIPOS_INMUEBLE)),
                campo("Presupuesto ($)", c.map(|x| num_txt(x.presupuesto)).unwrap_or_default()),
                selector("Tipo de negocio", &g(&|x| x.tipo_negocio.clone()), lista(&TIPOS_NEGOCIO)),
                selector("Asesor asignado", &g(&|x| x.asesor.clone()), asesores),
                selector("Fuente", &g(&|x| x.fuente.clone()), lista(&FUENTES_CRM)),
                selector("Etapa del pipeline", &g(&|x| x.etapa.clone()), lista(&ETAPAS_CRM)),
                campo("Fecha próximo contacto (AAAA-MM-DD)", g(&|x| x.proximo_contacto.clone())),
                campo("Notas", g(&|x| x.notas.clone())),
                campo("Inmueble sugerido", g(&|x| x.inmueble_sugerido.clone())),
                campo("Resultado", g(&|x| x.resultado.clone())),
                campo("Fecha de cierre (AAAA-MM-DD)", g(&|x| x.fecha_cierre.clone())),
            ])
        }
        M_GUARDIA => {
            let gu = &db.guardia[fila_db(db, m, idx.ok_or("La guardia solo se edita.")?)?];
            Ok(vec![
                campo(&format!("Abogado de guardia — {}", gu.dia), gu.abogado.clone()),
                campo("Cédula / Colegio", gu.cedula_colegio.clone()),
                campo("Teléfono", gu.telefono.clone()),
                campo("Correo electrónico", gu.correo.clone()),
                campo("Notas", gu.notas.clone()),
            ])
        }
        _ => Err("Este módulo no tiene formulario.".to_string()),
    }
}

fn exigir(valores: &[String], n: usize) -> Result<(), String> {
    if valores.len() == n { Ok(()) } else { Err("Formulario incompleto.".to_string()) }
}

/// Valida y guarda (en memoria; el llamador persiste con data::guardar).
pub fn guardar(db: &mut Database, m: i32, idx: Option<usize>, v: &[String]) -> Result<String, String> {
    match m {
        M_NEGOCIOS => {
            exigir(v, 16)?;
            let fecha = fecha_req(&v[0], "Fecha del negocio")?;
            let asesores = nombres_cortos(db);
            let asesor = en_lista(&v[1], &asesores, "Asesor")?;
            let mut formato = pct_opt(&v[2], "Formato %")?;
            if formato == 0.0 {
                formato = db.asesores.iter().find(|a| a.nombre_corto == asesor).map(|a| a.formato).unwrap_or(0.0);
            }
            if formato == 0.0 {
                return Err("Indique el Formato % (el asesor no tiene uno por defecto).".into());
            }
            let n = Negocio {
                id: 0,
                fecha,
                asesor,
                formato,
                punta_captadora: texto(&v[3]),
                punta_compradora: texto(&v[4]),
                punta_arrendataria: texto(&v[5]),
                asesor_otra_punta: texto(&v[6]),
                monto: num_req(&v[7], "Monto del negocio")?,
                tipo_inmueble: en_lista(&v[8], &lista(&TIPOS_INMUEBLE), "Tipo de inmueble")?,
                ubicacion: texto(&v[9]),
                canon: num_opt(&v[10], "Canon")?,
                tipo_negocio: en_lista(&v[11], &lista(&TIPOS_NEGOCIO), "Tipo de negocio")?,
                fin: en_lista(&v[12], &lista(&FINES), "Fin / propósito")?,
                comision_total: num_req(&v[13], "Comisión total")?,
                origen: texto(&v[14]),
                notas: texto(&v[15]),
            };
            match idx {
                Some(i) => {
                    let k = fila_db(db, m, i)?;
                    let id = db.negocios[k].id;
                    db.negocios[k] = Negocio { id, ..n };
                    Ok("Negocio actualizado.".into())
                }
                None => {
                    let id = db.negocios.iter().map(|x| x.id).max().unwrap_or(0) + 1;
                    db.negocios.push(Negocio { id, ..n });
                    Ok(format!("Negocio #{} guardado.", id))
                }
            }
        }
        M_ASESORES => {
            exigir(v, 7)?;
            let nombre_completo = texto(&v[0]);
            let corto = texto(&v[1]);
            if nombre_completo.is_empty() || corto.is_empty() {
                return Err("El nombre completo y el nombre corto son obligatorios.".into());
            }
            let actual = match idx {
                Some(i) => Some(fila_db(db, m, i)?),
                None => None,
            };
            if db.asesores.iter().enumerate().any(|(k, a)| Some(k) != actual && normalizar(&a.nombre_corto) == normalizar(&corto)) {
                return Err("Ya existe un asesor con ese nombre corto.".into());
            }
            let cargo = en_lista(&v[3], &lista(&CARGOS), "Cargo")?;
            let formato = pct_opt(&v[4], "Formato %")?;
            if cargo == "Agente Inmobiliario" && formato == 0.0 {
                return Err("Un agente inmobiliario necesita su Formato % (ej. 50%).".into());
            }
            let fecha_ingreso = fecha_opt(&v[5], "Fecha de ingreso")?;
            match actual {
                Some(k) => {
                    let viejo = db.asesores[k].nombre_corto.clone();
                    let codigo = db.asesores[k].codigo.clone();
                    db.asesores[k] = Asesor {
                        codigo, nombre_completo, nombre_corto: corto.clone(), cedula: texto(&v[2]), cargo, formato,
                        fecha_ingreso, telefono: texto(&v[6]),
                    };
                    if viejo != corto {
                        // El nombre corto es la "llave" en las demas hojas: se renombra en todas.
                        for n in db.negocios.iter_mut().filter(|n| n.asesor == viejo) { n.asesor = corto.clone(); }
                        for p in db.inventario.iter_mut().filter(|p| p.asesor == viejo) { p.asesor = corto.clone(); }
                        for c in db.clientes.iter_mut().filter(|c| c.asesor == viejo) { c.asesor = corto.clone(); }
                    }
                    Ok("Asesor actualizado.".into())
                }
                None => {
                    let sig = db
                        .asesores
                        .iter()
                        .filter_map(|a| a.codigo.trim_start_matches("AS-").parse::<u32>().ok())
                        .max()
                        .unwrap_or(0)
                        + 1;
                    let codigo = format!("AS-{:03}", sig);
                    db.asesores.push(Asesor {
                        codigo: codigo.clone(), nombre_completo, nombre_corto: corto, cedula: texto(&v[2]), cargo,
                        formato, fecha_ingreso, telefono: texto(&v[6]),
                    });
                    Ok(format!("Asesor {} guardado.", codigo))
                }
            }
        }
        M_MENSUALIDAD => {
            exigir(v, 1)?;
            let (cod, mes) = celdas_mensualidad(db, &mes_actual())
                .get(idx.ok_or("La mensualidad solo se edita.")?)
                .cloned()
                .ok_or("La fila seleccionada ya no existe.")?;
            let t = v[0].trim();
            let dia = if t.is_empty() {
                None
            } else {
                let d: u32 = t.parse().map_err(|_| "El día de pago debe ser un número del 1 al 31.".to_string())?;
                if !(1..=31).contains(&d) {
                    return Err("El día de pago debe estar entre 1 y 31.".into());
                }
                Some(d)
            };
            fijar_pago(db, &cod, &mes, dia);
            Ok(format!("Mensualidad: {}.", estatus_mensualidad(dia)))
        }
        M_FINANZAS => {
            exigir(v, 4)?;
            let mes = periodo().get(idx.ok_or("Las finanzas solo se editan.")?).cloned().ok_or("La fila TOTALES no se edita.")?;
            let e = EgresoMes {
                mes: mes.clone(),
                arriendo: num_opt(&v[0], "Arrendamiento")?,
                servicios: num_opt(&v[1], "Servicios")?,
                marketing: num_opt(&v[2], "Marketing")?,
                nomina: num_opt(&v[3], "Nómina")?,
            };
            db.egresos.retain(|x| x.mes != mes);
            db.egresos.push(e);
            db.egresos.sort_by(|a, b| a.mes.cmp(&b.mes));
            Ok("Egresos actualizados.".into())
        }
        M_INVENTARIO => {
            exigir(v, 15)?;
            let asesores = nombres_cortos(db);
            let p = Propiedad {
                id: 0,
                fecha: fecha_req(&v[0], "Fecha de captación")?,
                asesor: en_lista(&v[1], &asesores, "Asesor captador")?,
                tipo_inmueble: en_lista(&v[2], &lista(&TIPOS_INMUEBLE), "Tipo de inmueble")?,
                ubicacion: texto(&v[3]),
                precio_venta: num_opt(&v[4], "Precio de venta")?,
                canon: num_opt(&v[5], "Canon")?,
                tipo_operacion: en_lista(&v[6], &lista(&TIPOS_NEGOCIO), "Tipo de operación")?,
                estado: en_lista(&v[7], &lista(&ESTADOS_INV), "Estado")?,
                propietario: texto(&v[8]),
                telefono: texto(&v[9]),
                habitaciones: texto(&v[10]),
                banos: texto(&v[11]),
                m2: texto(&v[12]),
                caracteristicas: texto(&v[13]),
                notas: texto(&v[14]),
            };
            match idx {
                Some(i) => {
                    let k = fila_db(db, m, i)?;
                    let id = db.inventario[k].id;
                    db.inventario[k] = Propiedad { id, ..p };
                    Ok("Propiedad actualizada.".into())
                }
                None => {
                    let id = db.inventario.iter().map(|x| x.id).max().unwrap_or(0) + 1;
                    db.inventario.push(Propiedad { id, ..p });
                    Ok(format!("Propiedad #{} guardada.", id))
                }
            }
        }
        M_CRM => {
            exigir(v, 16)?;
            if v[1].trim().is_empty() {
                return Err("El nombre del cliente es obligatorio.".into());
            }
            let asesores = nombres_cortos(db);
            let c = Cliente {
                id: 0,
                fecha_contacto: fecha_req(&v[0], "Fecha de contacto")?,
                nombre: texto(&v[1]),
                telefono: texto(&v[2]),
                email: texto(&v[3]),
                zona: texto(&v[4]),
                tipo_inmueble: en_lista(&v[5], &lista(&TIPOS_INMUEBLE), "Tipo de inmueble")?,
                presupuesto: num_opt(&v[6], "Presupuesto")?,
                tipo_negocio: en_lista(&v[7], &lista(&TIPOS_NEGOCIO), "Tipo de negocio")?,
                asesor: en_lista(&v[8], &asesores, "Asesor asignado")?,
                fuente: en_lista(&v[9], &lista(&FUENTES_CRM), "Fuente")?,
                etapa: en_lista(&v[10], &lista(&ETAPAS_CRM), "Etapa")?,
                proximo_contacto: fecha_opt(&v[11], "Fecha próximo contacto")?,
                notas: texto(&v[12]),
                inmueble_sugerido: texto(&v[13]),
                resultado: texto(&v[14]),
                fecha_cierre: fecha_opt(&v[15], "Fecha de cierre")?,
            };
            match idx {
                Some(i) => {
                    let k = fila_db(db, m, i)?;
                    let id = db.clientes[k].id;
                    db.clientes[k] = Cliente { id, ..c };
                    Ok("Cliente actualizado.".into())
                }
                None => {
                    let id = db.clientes.iter().map(|x| x.id).max().unwrap_or(0) + 1;
                    db.clientes.push(Cliente { id, ..c });
                    Ok(format!("Cliente #{} guardado.", id))
                }
            }
        }
        M_GUARDIA => {
            exigir(v, 5)?;
            if v[0].trim().is_empty() {
                return Err("El abogado de guardia es obligatorio.".into());
            }
            let k = fila_db(db, m, idx.ok_or("La guardia solo se edita.")?)?;
            let dia = db.guardia[k].dia.clone();
            db.guardia[k] = Guardia {
                dia,
                abogado: texto(&v[0]),
                cedula_colegio: texto(&v[1]),
                telefono: texto(&v[2]),
                correo: texto(&v[3]),
                notas: texto(&v[4]),
            };
            Ok("Guardia actualizada.".into())
        }
        _ => Err("Este módulo no se edita.".into()),
    }
}

pub fn eliminar(db: &mut Database, m: i32, idx: usize) -> Result<(), String> {
    match m {
        M_NEGOCIOS => {
            let k = fila_db(db, m, idx)?;
            db.negocios.remove(k);
            Ok(())
        }
        M_ASESORES => {
            let k = fila_db(db, m, idx)?;
            let a = db.asesores[k].clone();
            if db.negocios.iter().any(|n| n.asesor == a.nombre_corto) {
                return Err(format!("No se puede eliminar a {}: tiene negocios registrados.", a.nombre_corto));
            }
            db.asesores.remove(k);
            db.mensualidad.retain(|p| p.codigo != a.codigo);
            Ok(())
        }
        M_INVENTARIO => {
            let k = fila_db(db, m, idx)?;
            db.inventario.remove(k);
            Ok(())
        }
        M_CRM => {
            let k = fila_db(db, m, idx)?;
            db.clientes.remove(k);
            Ok(())
        }
        _ => Err("Este módulo no permite eliminar filas.".into()),
    }
}
