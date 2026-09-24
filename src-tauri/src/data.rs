// ============================================================================
// data.rs
// Carga y guarda cada hoja como un .csv dentro de <carpeta de datos>/kavela/.
// Formato CSV simple (sin comillas). Al guardar, las comas y saltos de linea
// dentro de un campo se reemplazan (',' -> ';') para no romper el formato.
// La PRIMERA vez que se abre la app (si no hay CSV) se cargan los datos
// iniciales de seed.rs (tomados del Excel sistema_Kavela_1_1.xlsx).
// Los CSV del sistema anterior (carpeta data/ sin "kavela/") no se tocan.
// ============================================================================

use crate::models::*;
use crate::seed;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::sync::OnceLock;

static BASE_DIR: OnceLock<PathBuf> = OnceLock::new();

pub fn inicializar_dir(dir: PathBuf) {
    let _ = BASE_DIR.set(dir);
}

fn base_dir() -> String {
    let base = match BASE_DIR.get() {
        Some(p) => p.to_string_lossy().to_string(),
        None => "data".to_string(),
    };
    format!("{}/kavela", base)
}

fn ruta(nombre: &str) -> String {
    format!("{}/{}.csv", base_dir(), nombre)
}

fn parsear(contenido: &str) -> Vec<Vec<String>> {
    contenido
        .lines()
        .skip(1)
        .filter(|l| !l.trim().is_empty())
        .map(|l| l.split(',').map(|s| s.trim().to_string()).collect())
        .collect()
}

fn leer_filas(nombre: &str) -> Vec<Vec<String>> {
    match fs::read_to_string(ruta(nombre)) {
        Ok(c) => parsear(&c),
        Err(_) => Vec::new(),
    }
}

fn limpiar(s: &str) -> String {
    s.replace(',', ";").replace(['\n', '\r'], " ").trim().to_string()
}

fn escribir_csv(nombre: &str, encabezado: &str, filas: Vec<Vec<String>>) -> std::io::Result<()> {
    fs::create_dir_all(base_dir())?;
    let mut f = fs::File::create(ruta(nombre))?;
    writeln!(f, "{}", encabezado)?;
    for fila in filas {
        let l: Vec<String> = fila.iter().map(|c| limpiar(c)).collect();
        writeln!(f, "{}", l.join(","))?;
    }
    Ok(())
}

fn g(r: &[String], i: usize) -> String {
    r.get(i).cloned().unwrap_or_default()
}
fn pf(r: &[String], i: usize) -> f64 {
    r.get(i).map(|s| s.trim().replace(',', ".").parse::<f64>().unwrap_or(0.0)).unwrap_or(0.0)
}
fn pu(r: &[String], i: usize) -> u32 {
    r.get(i).map(|s| s.trim().parse::<u32>().unwrap_or(0)).unwrap_or(0)
}
fn n(v: f64) -> String {
    if v == v.trunc() { format!("{}", v as i64) } else { format!("{}", v) }
}
/// Numero opcional: 0 se guarda como vacio (asi queda igual que las celdas vacias del Excel).
fn n0(v: f64) -> String {
    if v == 0.0 { String::new() } else { n(v) }
}

/// True si ya existe la carpeta de datos de Kavela (si no, se sembrara el seed).
pub fn datos_existen() -> bool {
    fs::metadata(ruta("asesores")).is_ok()
}

/// Escribe los datos iniciales solo si nunca se guardo nada.
pub fn sembrar_si_hace_falta() {
    if datos_existen() {
        return;
    }
    let _ = fs::create_dir_all(base_dir());
    for (nombre, contenido) in [
        ("asesores", seed::ASESORES),
        ("negocios", seed::NEGOCIOS),
        ("mensualidad", seed::MENSUALIDAD),
        ("egresos", seed::EGRESOS),
        ("inventario", seed::INVENTARIO),
        ("clientes", seed::CLIENTES),
        ("guardia", seed::GUARDIA),
    ] {
        let _ = fs::write(ruta(nombre), contenido);
    }
}

/// Base de datos con los datos iniciales del Excel (sin tocar el disco; la usan los tests).
#[allow(dead_code)]
pub fn desde_semilla() -> Database {
    construir(&|nombre| {
        parsear(match nombre {
            "asesores" => seed::ASESORES,
            "negocios" => seed::NEGOCIOS,
            "mensualidad" => seed::MENSUALIDAD,
            "egresos" => seed::EGRESOS,
            "inventario" => seed::INVENTARIO,
            "clientes" => seed::CLIENTES,
            _ => seed::GUARDIA,
        })
    })
}

pub fn cargar() -> Database {
    sembrar_si_hace_falta();
    construir(&leer_filas)
}

fn construir(leer_filas: &dyn Fn(&str) -> Vec<Vec<String>>) -> Database {
    Database {
        asesores: leer_filas("asesores")
            .iter()
            .filter(|r| !g(r, 0).is_empty())
            .map(|r| Asesor {
                codigo: g(r, 0),
                nombre_completo: g(r, 1),
                nombre_corto: g(r, 2),
                cedula: g(r, 3),
                cargo: g(r, 4),
                formato: pf(r, 5),
                fecha_ingreso: g(r, 6),
                telefono: g(r, 7),
            })
            .collect(),
        negocios: leer_filas("negocios")
            .iter()
            .filter(|r| pu(r, 0) > 0)
            .map(|r| Negocio {
                id: pu(r, 0),
                fecha: g(r, 1),
                asesor: g(r, 2),
                formato: pf(r, 3),
                punta_captadora: g(r, 4),
                punta_compradora: g(r, 5),
                punta_arrendataria: g(r, 6),
                asesor_otra_punta: g(r, 7),
                monto: pf(r, 8),
                tipo_inmueble: g(r, 9),
                ubicacion: g(r, 10),
                canon: pf(r, 11),
                tipo_negocio: g(r, 12),
                fin: g(r, 13),
                comision_total: pf(r, 14),
                origen: g(r, 15),
                notas: g(r, 16),
            })
            .collect(),
        mensualidad: leer_filas("mensualidad")
            .iter()
            .filter(|r| !g(r, 0).is_empty())
            .map(|r| PagoMensualidad { codigo: g(r, 0), mes: g(r, 1), dia: pu(r, 2) })
            .collect(),
        egresos: leer_filas("egresos")
            .iter()
            .filter(|r| !g(r, 0).is_empty())
            .map(|r| EgresoMes {
                mes: g(r, 0),
                arriendo: pf(r, 1),
                servicios: pf(r, 2),
                marketing: pf(r, 3),
                nomina: pf(r, 4),
            })
            .collect(),
        inventario: leer_filas("inventario")
            .iter()
            .filter(|r| pu(r, 0) > 0)
            .map(|r| Propiedad {
                id: pu(r, 0),
                fecha: g(r, 1),
                asesor: g(r, 2),
                tipo_inmueble: g(r, 3),
                ubicacion: g(r, 4),
                precio_venta: pf(r, 5),
                canon: pf(r, 6),
                tipo_operacion: g(r, 7),
                estado: g(r, 8),
                propietario: g(r, 9),
                telefono: g(r, 10),
                habitaciones: g(r, 11),
                banos: g(r, 12),
                m2: g(r, 13),
                caracteristicas: g(r, 14),
                notas: g(r, 15),
            })
            .collect(),
        clientes: leer_filas("clientes")
            .iter()
            .filter(|r| pu(r, 0) > 0)
            .map(|r| Cliente {
                id: pu(r, 0),
                fecha_contacto: g(r, 1),
                nombre: g(r, 2),
                telefono: g(r, 3),
                email: g(r, 4),
                zona: g(r, 5),
                tipo_inmueble: g(r, 6),
                presupuesto: pf(r, 7),
                tipo_negocio: g(r, 8),
                asesor: g(r, 9),
                fuente: g(r, 10),
                etapa: g(r, 11),
                proximo_contacto: g(r, 12),
                notas: g(r, 13),
                inmueble_sugerido: g(r, 14),
                resultado: g(r, 15),
                fecha_cierre: g(r, 16),
            })
            .collect(),
        guardia: leer_filas("guardia")
            .iter()
            .filter(|r| !g(r, 0).is_empty())
            .map(|r| Guardia {
                dia: g(r, 0),
                abogado: g(r, 1),
                cedula_colegio: g(r, 2),
                telefono: g(r, 3),
                correo: g(r, 4),
                notas: g(r, 5),
            })
            .collect(),
    }
}

pub fn guardar(db: &Database) -> std::io::Result<()> {
    escribir_csv(
        "asesores",
        "codigo,nombre_completo,nombre_corto,cedula,cargo,formato,fecha_ingreso,telefono",
        db.asesores
            .iter()
            .map(|a| {
                vec![
                    a.codigo.clone(), a.nombre_completo.clone(), a.nombre_corto.clone(),
                    a.cedula.clone(), a.cargo.clone(), n0(a.formato),
                    a.fecha_ingreso.clone(), a.telefono.clone(),
                ]
            })
            .collect(),
    )?;
    escribir_csv(
        "negocios",
        "id,fecha,asesor,formato,punta_captadora,punta_compradora,punta_arrendataria,asesor_otra_punta,monto,tipo_inmueble,ubicacion,canon,tipo_negocio,fin,comision_total,origen,notas",
        db.negocios
            .iter()
            .map(|x| {
                vec![
                    x.id.to_string(), x.fecha.clone(), x.asesor.clone(), n(x.formato),
                    x.punta_captadora.clone(), x.punta_compradora.clone(),
                    x.punta_arrendataria.clone(), x.asesor_otra_punta.clone(), n(x.monto),
                    x.tipo_inmueble.clone(), x.ubicacion.clone(), n0(x.canon),
                    x.tipo_negocio.clone(), x.fin.clone(), n(x.comision_total),
                    x.origen.clone(), x.notas.clone(),
                ]
            })
            .collect(),
    )?;
    escribir_csv(
        "mensualidad",
        "codigo,mes,dia",
        db.mensualidad
            .iter()
            .map(|p| vec![p.codigo.clone(), p.mes.clone(), p.dia.to_string()])
            .collect(),
    )?;
    escribir_csv(
        "egresos",
        "mes,arriendo,servicios,marketing,nomina",
        db.egresos
            .iter()
            .map(|e| vec![e.mes.clone(), n(e.arriendo), n(e.servicios), n(e.marketing), n(e.nomina)])
            .collect(),
    )?;
    escribir_csv(
        "inventario",
        "id,fecha,asesor,tipo_inmueble,ubicacion,precio_venta,canon,tipo_operacion,estado,propietario,telefono,habitaciones,banos,m2,caracteristicas,notas",
        db.inventario
            .iter()
            .map(|p| {
                vec![
                    p.id.to_string(), p.fecha.clone(), p.asesor.clone(), p.tipo_inmueble.clone(),
                    p.ubicacion.clone(), n0(p.precio_venta), n0(p.canon), p.tipo_operacion.clone(),
                    p.estado.clone(), p.propietario.clone(), p.telefono.clone(),
                    p.habitaciones.clone(), p.banos.clone(), p.m2.clone(),
                    p.caracteristicas.clone(), p.notas.clone(),
                ]
            })
            .collect(),
    )?;
    escribir_csv(
        "clientes",
        "id,fecha_contacto,nombre,telefono,email,zona,tipo_inmueble,presupuesto,tipo_negocio,asesor,fuente,etapa,proximo_contacto,notas,inmueble_sugerido,resultado,fecha_cierre",
        db.clientes
            .iter()
            .map(|c| {
                vec![
                    c.id.to_string(), c.fecha_contacto.clone(), c.nombre.clone(), c.telefono.clone(),
                    c.email.clone(), c.zona.clone(), c.tipo_inmueble.clone(), n0(c.presupuesto),
                    c.tipo_negocio.clone(), c.asesor.clone(), c.fuente.clone(), c.etapa.clone(),
                    c.proximo_contacto.clone(), c.notas.clone(), c.inmueble_sugerido.clone(),
                    c.resultado.clone(), c.fecha_cierre.clone(),
                ]
            })
            .collect(),
    )?;
    escribir_csv(
        "guardia",
        "dia,abogado,cedula_colegio,telefono,correo,notas",
        db.guardia
            .iter()
            .map(|x| {
                vec![
                    x.dia.clone(), x.abogado.clone(), x.cedula_colegio.clone(),
                    x.telefono.clone(), x.correo.clone(), x.notas.clone(),
                ]
            })
            .collect(),
    )?;
    Ok(())
}
