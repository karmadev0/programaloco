// ============================================================================
// models.rs
// Modelo de datos de KAVELA (reemplaza al modelo del Excel anterior).
// Solo se guardan los datos que se digitan. Todo lo que en el Excel era
// FORMULA (comision asesor, comision oficina, mes, año, estado del asesor,
// estatus de mensualidad, % de probabilidad del CRM, rankings, puntajes,
// finanzas mensuales...) se recalcula siempre en logic.rs.
// ============================================================================

/// Hoja "Asesores" (parte digitada).
#[derive(Debug, Clone)]
pub struct Asesor {
    pub codigo: String,          // "AS-001"
    pub nombre_completo: String, // "CARLOS ENRIQUE CACERES RODRIGUEZ"
    pub nombre_corto: String,    // "Carlos Caceres" (el que se usa en Registro_Negocios)
    pub cedula: String,
    pub cargo: String,           // "Broker/Abg" | "Agente Inmobiliario" | "Administrativo"
    pub formato: f64,            // 0.5 = 50%  (0 = no aplica)
    pub fecha_ingreso: String,   // "2026-04-21" o ""
    pub telefono: String,
}

/// Hoja "Registro_Negocios": UNICA hoja de entrada de datos del Excel.
#[derive(Debug, Clone)]
pub struct Negocio {
    pub id: u32,
    pub fecha: String,            // "2026-04-17"
    pub asesor: String,           // nombre_corto del asesor
    pub formato: f64,             // % de la comision total que se lleva el asesor
    pub punta_captadora: String,
    pub punta_compradora: String,
    pub punta_arrendataria: String,
    pub asesor_otra_punta: String,
    pub monto: f64,
    pub tipo_inmueble: String,
    pub ubicacion: String,
    pub canon: f64,
    pub tipo_negocio: String,     // "Venta" | "Alquiler" | ...
    pub fin: String,              // "Residencial" | "Comercial" | "Agro" ...
    pub comision_total: f64,
    pub origen: String,
    pub notas: String,
}

/// Hoja "Mensualidad": un pago = (asesor, mes, dia del mes en que pago).
/// Si no existe el registro, ese mes esta "Pendiente".
#[derive(Debug, Clone)]
pub struct PagoMensualidad {
    pub codigo: String, // "AS-003"
    pub mes: String,    // "2026-04"
    pub dia: u32,
}

/// Hoja "Finanzas_Gastos": egresos digitados de un mes (celdas amarillas).
#[derive(Debug, Clone)]
pub struct EgresoMes {
    pub mes: String, // "2026-04"
    pub arriendo: f64,
    pub servicios: f64,
    pub marketing: f64,
    pub nomina: f64,
}

/// Hoja "Inventario": propiedades en cartera.
#[derive(Debug, Clone)]
pub struct Propiedad {
    pub id: u32,
    pub fecha: String,
    pub asesor: String, // captador (nombre_corto)
    pub tipo_inmueble: String,
    pub ubicacion: String,
    pub precio_venta: f64,
    pub canon: f64,
    pub tipo_operacion: String,
    pub estado: String,
    pub propietario: String,
    pub telefono: String,
    pub habitaciones: String,
    pub banos: String,
    pub m2: String,
    pub caracteristicas: String,
    pub notas: String,
}

/// Hoja "CRM_Clientes".
#[derive(Debug, Clone)]
pub struct Cliente {
    pub id: u32,
    pub fecha_contacto: String,
    pub nombre: String,
    pub telefono: String,
    pub email: String,
    pub zona: String,
    pub tipo_inmueble: String,
    pub presupuesto: f64,
    pub tipo_negocio: String,
    pub asesor: String,
    pub fuente: String,
    pub etapa: String,
    pub proximo_contacto: String,
    pub notas: String,
    pub inmueble_sugerido: String,
    pub resultado: String,
    pub fecha_cierre: String,
}

/// Hoja "Guardia_Juridica": un abogado por dia (lunes a viernes).
#[derive(Debug, Clone)]
pub struct Guardia {
    pub dia: String,
    pub abogado: String,
    pub cedula_colegio: String,
    pub telefono: String,
    pub correo: String,
    pub notas: String,
}

/// Todo el "libro" en memoria.
#[derive(Debug, Clone, Default)]
pub struct Database {
    pub asesores: Vec<Asesor>,
    pub negocios: Vec<Negocio>,
    pub mensualidad: Vec<PagoMensualidad>,
    pub egresos: Vec<EgresoMes>,
    pub inventario: Vec<Propiedad>,
    pub clientes: Vec<Cliente>,
    pub guardia: Vec<Guardia>,
}
