mod data;
mod logic;
mod models;
mod modulos;
mod seed;
#[cfg(test)]
mod verificacion;

use modulos::*;
use slint::{ModelRc, SharedString, VecModel};
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

slint::include_modules!();

/// Carpeta de datos del usuario segun el sistema operativo.
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

const PANTALLA_AYUDA: i32 = 12;

// ---------------------------------------------------------------------
// Conversiones a modelos de Slint
// ---------------------------------------------------------------------

fn textos(v: Vec<String>) -> ModelRc<SharedString> {
    ModelRc::new(VecModel::from(v.into_iter().map(SharedString::from).collect::<Vec<_>>()))
}

fn filas(f: Vec<Vec<String>>) -> ModelRc<Fila> {
    ModelRc::new(VecModel::from(f.into_iter().map(|c| Fila { celdas: textos(c) }).collect::<Vec<_>>()))
}

fn kpis(v: Vec<(String, String)>) -> ModelRc<Kpi> {
    ModelRc::new(VecModel::from(
        v.into_iter().map(|(e, val)| Kpi { etiqueta: e.into(), valor: val.into() }).collect::<Vec<_>>(),
    ))
}

// ---------------------------------------------------------------------
// Estado del formulario (los valores viven aqui, no en Slint, para que
// escribir en un campo nunca reconstruya el formulario y pierda el foco)
// ---------------------------------------------------------------------

struct Estado {
    modulo: i32,
    editando: Option<usize>,
    valores: Vec<String>,
}

// ---------------------------------------------------------------------
// Carga de pantallas
// ---------------------------------------------------------------------

fn cargar_dashboard(v: &AppWindow) {
    let db = data::cargar();
    let d = dashboard(&db);
    v.set_kpis(kpis(d.kpis));
    v.set_honor(kpis(d.honor));
    v.set_dash_headers(textos(d.headers));
    v.set_dash_rows(filas(d.filas));
}

fn cargar_graficos(v: &AppWindow) {
    let db = data::cargar();
    let g: Vec<Grafico> = graficos(&db)
        .into_iter()
        .map(|g| {
            let max = g.barras.iter().map(|b| b.1).fold(0.0_f64, f64::max);
            let barras: Vec<Barra> = g
                .barras
                .into_iter()
                .map(|(e, val, t)| Barra {
                    etiqueta: e.into(),
                    valor: if max > 0.0 { (val / max) as f32 } else { 0.0 },
                    texto: t.into(),
                })
                .collect();
            Grafico { titulo: g.titulo.into(), barras: ModelRc::new(VecModel::from(barras)) }
        })
        .collect();
    v.set_graficos(ModelRc::new(VecModel::from(g)));
}

fn cargar_vista(v: &AppWindow, m: i32) {
    let db = data::cargar();
    let vs = vista(&db, m);
    v.set_titulo(vs.titulo.into());
    v.set_ayuda(vs.ayuda.into());
    v.set_headers(textos(vs.headers));
    v.set_visibles(vs.visibles as i32);
    v.set_rows(filas(vs.filas));
    v.set_puede_crear(vs.puede_crear);
    v.set_puede_editar(vs.puede_editar);
    v.set_puede_eliminar(vs.puede_eliminar);
    v.set_mostrar_form(false);
    v.set_mensaje("".into());
}

fn cargar_pantalla(v: &AppWindow, m: i32) {
    match m {
        M_DASHBOARD => cargar_dashboard(v),
        M_GRAFICOS => cargar_graficos(v),
        PANTALLA_AYUDA => {}
        _ => cargar_vista(v, m),
    }
}

fn mostrar_form(v: &AppWindow, est: &Rc<RefCell<Estado>>, campos: Vec<CampoForm>, editando: Option<usize>) {
    {
        let mut e = est.borrow_mut();
        e.valores = campos.iter().map(|c| c.valor.clone()).collect();
        e.editando = editando;
    }
    let modelo: Vec<Campo> = campos
        .into_iter()
        .map(|c| Campo { etiqueta: c.etiqueta.into(), valor: c.valor.into(), opciones: textos(c.opciones) })
        .collect();
    v.set_campos(ModelRc::new(VecModel::from(modelo)));
    v.set_editando(editando.is_some());
    v.set_mensaje("".into());
    v.set_mostrar_form(true);
}

fn main() {
    let dir = resolver_data_dir();
    std::fs::create_dir_all(&dir).ok();
    data::inicializar_dir(dir.join("data"));

    let v = AppWindow::new().expect("no se pudo crear la ventana");
    let estado = Rc::new(RefCell::new(Estado { modulo: M_DASHBOARD, editando: None, valores: vec![] }));

    cargar_dashboard(&v);

    // --- navegacion ---
    {
        let debil = v.as_weak();
        let est = estado.clone();
        v.on_ir_a(move |n| {
            if let Some(v) = debil.upgrade() {
                est.borrow_mut().modulo = n;
                v.set_pantalla(n);
                cargar_pantalla(&v, n);
            }
        });
    }

    // --- "Nuevo": abre el formulario vacio ---
    {
        let debil = v.as_weak();
        let est = estado.clone();
        v.on_nuevo(move || {
            if let Some(v) = debil.upgrade() {
                let m = est.borrow().modulo;
                let db = data::cargar();
                match campos(&db, m, None) {
                    Ok(c) => mostrar_form(&v, &est, c, None),
                    Err(e) => v.set_mensaje(e.into()),
                }
            }
        });
    }

    // --- "Editar": abre el formulario con los datos de la fila ---
    {
        let debil = v.as_weak();
        let est = estado.clone();
        v.on_editar(move |i| {
            if let Some(v) = debil.upgrade() {
                let m = est.borrow().modulo;
                let db = data::cargar();
                match campos(&db, m, Some(i as usize)) {
                    Ok(c) => mostrar_form(&v, &est, c, Some(i as usize)),
                    Err(e) => v.set_mensaje(e.into()),
                }
            }
        });
    }

    // --- cada tecla / seleccion del formulario ---
    {
        let est = estado.clone();
        v.on_campo_editado(move |i, texto| {
            if let Some(c) = est.borrow_mut().valores.get_mut(i as usize) {
                *c = texto.to_string();
            }
        });
    }

    // --- Guardar ---
    {
        let debil = v.as_weak();
        let est = estado.clone();
        v.on_guardar(move || {
            if let Some(v) = debil.upgrade() {
                let (m, editando, valores) = {
                    let e = est.borrow();
                    (e.modulo, e.editando, e.valores.clone())
                };
                let mut db = data::cargar();
                match modulos::guardar(&mut db, m, editando, &valores) {
                    Ok(msg) => match data::guardar(&db) {
                        Ok(()) => {
                            cargar_vista(&v, m);
                            v.set_mensaje(msg.into());
                        }
                        Err(e) => v.set_mensaje(format!("No se pudo guardar en disco: {}", e).into()),
                    },
                    // El formulario queda abierto con lo escrito, para corregir.
                    Err(e) => v.set_mensaje(format!("Error: {}", e).into()),
                }
            }
        });
    }

    // --- Cancelar ---
    {
        let debil = v.as_weak();
        v.on_cancelar(move || {
            if let Some(v) = debil.upgrade() {
                v.set_mostrar_form(false);
                v.set_mensaje("".into());
            }
        });
    }

    // --- Eliminar ---
    {
        let debil = v.as_weak();
        let est = estado.clone();
        v.on_eliminar(move |i| {
            if let Some(v) = debil.upgrade() {
                let m = est.borrow().modulo;
                let mut db = data::cargar();
                match modulos::eliminar(&mut db, m, i as usize) {
                    Ok(()) => match data::guardar(&db) {
                        Ok(()) => {
                            cargar_vista(&v, m);
                            v.set_mensaje("Registro eliminado.".into());
                        }
                        Err(e) => v.set_mensaje(format!("No se pudo guardar en disco: {}", e).into()),
                    },
                    Err(e) => v.set_mensaje(format!("Error: {}", e).into()),
                }
            }
        });
    }

    // --- Buscador de guardia juridica ---
    {
        let debil = v.as_weak();
        v.on_consultar_guardia(move |fecha| {
            if let Some(v) = debil.upgrade() {
                let db = data::cargar();
                v.set_guardia_resultado(consultar_guardia(&db, fecha.as_str()).into());
            }
        });
    }

    v.run().expect("error al correr la ventana");
}
