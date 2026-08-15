// ============================================================================
// models.rs
// Estructuras que representan cada fila de las hojas del Excel original.
// Los campos "calculados" del Excel (los que eran FORMULA=...) NO se guardan
// aqui: se recalculan siempre en logic.rs, igual que una celda de Excel se
// recalcula cada vez que abris el archivo. Asi el programa nunca puede
// quedar con datos derivados desincronizados de los datos base.
// ============================================================================

#[derive(Debug, Clone)]
pub struct Asesor {
    pub id: String,             // "AS-001"
    pub nombre: String,
    pub fecha_ingreso: String,  // "2024-01-15"
    pub talleres_asistidos: f64, // input manual (columna G de BD_Asesores)
}

#[derive(Debug, Clone)]
pub struct Captacion {
    pub cod_inmueble: String,   // "INM-101"
    pub tipo_propiedad: String,
    pub id_captador: String,    // AS-xxx
    pub fecha_captacion: String,
    pub precio_lista: f64,
    pub estatus: String,        // "Disponible" | "Cerrado" | ...
    pub publicado_web: bool,
    pub publicado_rrss: bool,
}

#[derive(Debug, Clone)]
pub struct Cierre {
    pub id: String,              // "CR-001"
    pub fecha_cierre: String,
    pub cod_inmueble: String,
    pub tipo_operacion: String,  // "Venta" | "Alquiler"
    pub monto_operacion: f64,
    pub id_captador: String,
    pub id_cerrador: String,
    pub pct_comision_total: f64, // ej. 0.05 = 5%
}

#[derive(Debug, Clone)]
pub struct TransaccionFinanciera {
    pub id: String,          // "TR-001"
    pub fecha: String,
    pub semana: String,
    pub tipo_flujo: String,  // "Ingreso" | "Egreso"
    pub categoria: String,
    pub monto: f64,
    pub estatus_pago: String, // "Pagado" | "Pendiente"
}

#[derive(Debug, Clone)]
pub struct Requerimiento {
    pub id: String,               // "REQ-001"
    pub cliente_buscador: String,
    pub asesor_cliente: String,
    pub tipo_operacion: String,
    pub zona_deseada: String,
    pub presupuesto_max: f64,
    pub inmueble_matcheado: String,
    pub precio_lista: f64,
    pub fecha_venc_exclusividad: String, // "2026-10-30" -> se calculan dias restantes
}

#[derive(Debug, Clone)]
pub struct ExpedienteLegal {
    pub cod_inmueble: String,
    pub propietario: String,
    pub titulo_propiedad: bool,
    pub cedula_rif: bool,
    pub ficha_catastral: bool,
    pub solvencia_municipal: bool,
    pub liberacion_hipoteca: String, // "SI" | "NO" | "N/A"
    pub borrador_contrato: String,   // "APROBADO" | "EN REVISIÓN" | "PENDIENTE"
    pub estatus_notaria: String,
}

#[derive(Debug, Clone)]
pub struct EmbudoAsesor {
    pub id_asesor: String,
    pub nombre_asesor: String,
    pub llamadas_realizadas: f64,
    pub citas_captacion: f64,
    pub visitas_guiadas: f64,
    pub ofertas_recibidas: f64,
    pub cierres_mes: f64,
}

#[derive(Debug, Clone)]
pub struct ReportePropietario {
    pub cod_inmueble: String,
    pub propietario: String,
    pub telefono: String,
    pub id_asesor: String,
    pub visitas_agendadas: f64,
    pub ofertas_recibidas: f64,
    pub canales_publicacion: String,
    pub notas: String,
    pub estatus_envio: String, // "ENVIADO" | "PENDIENTE"
}

/// Contenedor de todo el "libro" en memoria - equivalente al Workbook.
pub struct Database {
    pub asesores: Vec<Asesor>,
    pub captaciones: Vec<Captacion>,
    pub cierres: Vec<Cierre>,
    pub finanzas: Vec<TransaccionFinanciera>,
    pub matching: Vec<Requerimiento>,
    pub legal: Vec<ExpedienteLegal>,
    pub embudo: Vec<EmbudoAsesor>,
    pub reportes: Vec<ReportePropietario>,
}
