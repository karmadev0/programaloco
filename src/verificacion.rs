// ============================================================================
// verificacion.rs  (cargo test)
// Comprueba que el programa da los MISMOS numeros que el Excel de Kavela
// (valores tomados de las hojas Dashboard / Asesores / Finanzas_Gastos / TOP_Mensual).
// ============================================================================
use crate::{data, logic::*, models::*, modulos::*};
use std::sync::Once;

static INICIO: Once = Once::new();

fn db_semilla() -> Database {
    data::desde_semilla()
}

#[test]
fn t_totales_del_dashboard() {
    let db = db_semilla();
    let k = calcular_dashboard(&db);
    assert_eq!(k.total_negocios, 22); // el Excel mostraba 43 por una formula duplicada
    assert_eq!(k.ventas, 8);
    assert_eq!(k.alquileres, 14);
    assert_eq!(k.volumen, 385115.0);
    assert_eq!(k.comision_total, 13624.0);
    assert_eq!(k.comision_oficina, 6777.0);
    assert_eq!(k.top_producer, "Karina Rodriguez");
}

#[test]
fn t_desempeno_por_asesor() {
    let db = db_semilla();
    let d = calcular_desempeno(&db);
    let k = d.iter().find(|x| x.codigo == "AS-002").unwrap();
    assert_eq!((k.negocios, k.ventas, k.alquileres, k.captaciones), (9, 5, 4, 8));
    assert_eq!(k.volumen, 225390.0);
    assert_eq!(k.com_asesor, 2995.0);
    assert_eq!(k.puntaje, 116193.5);
    assert_eq!(k.ranking, 1);
    let v = d.iter().find(|x| x.codigo == "AS-011").unwrap();
    assert_eq!((v.puntaje, v.ranking, v.captaciones), (34730.75, 2, 3));
    let s = d.iter().find(|x| x.codigo == "AS-010").unwrap();
    assert_eq!((s.com_asesor, s.puntaje, s.ranking), (582.0, 20657.1, 4));
    let c = d.iter().find(|x| x.codigo == "AS-001").unwrap();
    assert_eq!((c.puntaje, c.ranking, c.estado.as_str()), (0.0, 10, "Activo"));
    assert_eq!(d.iter().find(|x| x.codigo == "AS-009").unwrap().estado, "Sin negocios");
}

#[test]
fn t_top_mensual() {
    let db = db_semilla();
    let t = top_mensual(&db);
    assert_eq!((t[0].top_producer.as_str(), t[0].volumen, t[0].negocios_mes, t[0].com_oficina), ("Karina Rodriguez", 144720.0, 5, 2747.5));
    assert_eq!((t[0].segundo.as_str(), t[0].vol2), ("Valentina Paradiso", 67000.0));
    assert_eq!((t[1].top_producer.as_str(), t[1].negocios_mes, t[1].com_oficina), ("Yule Cardenas", 4, 1292.5));
    assert_eq!((t[4].negocios_mes, t[4].com_oficina), (8, 1617.0));
    assert_eq!(t[5].top_producer, "-"); // Sep 2026 sin negocios
    assert_eq!(t[0].top_captador, "Karina Rodriguez");
}

#[test]
fn t_finanzas_y_mensualidad() {
    let db = db_semilla();
    let f = finanzas_mensuales(&db);
    // Abr 2026: comisiones 1600+700+1300+220+1675 = 5495; mensualidades 11 x $10; egresos 665
    assert_eq!(f[0].ingr_comisiones, 5495.0);
    assert_eq!(f[0].ingr_mensualidad, 110.0);
    assert_eq!(f[0].total_egresos, 665.0);
    assert_eq!(f[0].utilidad, 4940.0);
    assert_eq!(f[5].ingr_mensualidad, 0.0); // Sep 2026 sin pagos
    assert_eq!(estatus_mensualidad(None), "Pendiente");
    assert_eq!(estatus_mensualidad(Some(5)), "Pagado a Tiempo");
    assert_eq!(estatus_mensualidad(Some(15)), "Pagado con Mora");
    assert_eq!(estatus_mensualidad(Some(16)), "Mora Severa");
}

#[test]
fn t_crm_y_guardia() {
    let db = db_semilla();
    assert_eq!(prob_cierre("Visita"), Some(0.6));
    assert_eq!(prob_cierre("Perdido"), Some(0.0));
    // 2026-09-01 fue martes
    assert_eq!(abogado_de_guardia(&db, "2026-09-01"), "Martes: David Colina");
    assert!(abogado_de_guardia(&db, "2026-09-05").contains("no laborable"));
    assert_eq!(abogado_de_guardia(&db, "01/09/2026"), "Martes: David Colina");
}

#[test]
fn t_crud_negocio_y_asesor() {
    let mut db = db_semilla();
    let campos_nuevo = campos(&db, M_NEGOCIOS, None).unwrap();
    let mut v: Vec<String> = campos_nuevo.iter().map(|c| c.valor.clone()).collect();
    v[0] = "2026-09-10".into();
    v[1] = "Estefanny Cardenas".into();
    v[7] = "1000".into();
    v[13] = "100".into();
    assert!(guardar(&mut db, M_NEGOCIOS, None, &v).is_ok());
    assert_eq!(db.negocios.len(), 23);
    let n = db.negocios.last().unwrap();
    assert_eq!((n.id, n.formato, comision_asesor(n), comision_oficina(n)), (23, 0.5, 50.0, 50.0));
    // el mas reciente aparece primero en la lista
    assert_eq!(vista(&db, M_NEGOCIOS).filas[0][0], "23");
    // validaciones
    v[13] = "abc".into();
    assert!(guardar(&mut db, M_NEGOCIOS, None, &v).is_err());
    // renombrar un asesor arrastra sus negocios
    let mut a: Vec<String> = campos(&db, M_ASESORES, Some(2)).unwrap().iter().map(|c| c.valor.clone()).collect();
    a[1] = "Estefanny Cardenas G".into();
    guardar(&mut db, M_ASESORES, Some(2), &a).unwrap();
    assert!(db.negocios.iter().any(|n| n.asesor == "Estefanny Cardenas G"));
    // no se puede borrar un asesor con negocios
    assert!(eliminar(&mut db, M_ASESORES, 2).is_err());
    // mensualidad: poner y quitar un pago
    let fila = vista(&db, M_MENSUALIDAD).filas;
    assert!(!fila.is_empty());
    guardar(&mut db, M_MENSUALIDAD, Some(0), &["20".to_string()]).unwrap();
    assert_eq!(vista(&db, M_MENSUALIDAD).filas[0][4], "Mora Severa");
    guardar(&mut db, M_MENSUALIDAD, Some(0), &["".to_string()]).unwrap();
    assert_eq!(vista(&db, M_MENSUALIDAD).filas[0][4], "Pendiente");
}

#[test]
fn t_modos_respaldo_e_importacion() {
    // Un solo test para todo lo que toca disco (el modo es un estado global).
    INICIO.call_once(|| {
        let dir = std::env::temp_dir().join(format!("inmocore_test_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        data::inicializar_dir(dir);
    });
    let tmp = std::env::temp_dir().join(format!("inmocore_exp_{}", std::process::id()));
    let tmp = tmp.to_string_lossy().to_string();

    // REAL: arranca totalmente en blanco y todo funciona sin datos
    data::set_demo(false);
    let real = data::cargar();
    assert!(real.asesores.is_empty() && real.negocios.is_empty());
    assert_eq!(real.guardia.len(), 5);
    assert_eq!(calcular_dashboard(&real).total_negocios, 0);
    assert_eq!(calcular_dashboard(&real).top_producer, "-");
    assert!(campos(&real, M_NEGOCIOS, None).is_err()); // pide crear un asesor primero
    assert!(data::restaurar_demo().is_err());

    // DEMO: datos de ejemplo, en carpeta aparte
    data::set_demo(true);
    let mut db = data::cargar();
    assert_eq!((db.asesores.len(), db.negocios.len(), db.mensualidad.len()), (14, 22, 55));
    db.negocios.pop();
    data::guardar(&db).unwrap();
    assert_eq!(data::cargar().negocios.len(), 21);
    data::set_demo(false);
    assert!(data::cargar().negocios.is_empty()); // lo de la demo no toca lo real

    // exportar -> vaciar -> importar
    data::set_demo(true);
    assert!(data::exportar(&tmp).is_ok());
    assert!(data::vaciar().is_ok());
    assert!(data::cargar().negocios.is_empty());
    assert!(data::importar("/no/existe").is_err());
    assert!(data::importar(&tmp).is_ok());
    assert_eq!(data::cargar().negocios.len(), 21);
    assert!(data::restaurar_demo().is_ok());
    assert_eq!(data::cargar().negocios.len(), 22);
    data::set_demo(false);

    // AISLAMIENTO: nada de lo que se haga en Demo puede tocar la carpeta del modo Real.
    fn foto(dir: &str) -> Vec<(String, Vec<u8>)> {
        let mut v: Vec<(String, Vec<u8>)> = std::fs::read_dir(dir)
            .unwrap()
            .map(|e| {
                let p = e.unwrap().path();
                (p.file_name().unwrap().to_string_lossy().to_string(), std::fs::read(&p).unwrap())
            })
            .collect();
        v.sort();
        v
    }
    let mut real = data::cargar();
    real.asesores.push(Asesor {
        codigo: "AS-001".into(), nombre_completo: "PRUEBA REAL".into(), nombre_corto: "Prueba Real".into(),
        cedula: "1".into(), cargo: "Broker/Abg".into(), formato: 0.0, fecha_ingreso: String::new(), telefono: String::new(),
    });
    data::guardar(&real).unwrap();
    let dir_real = data::carpeta_datos();
    let antes = foto(&dir_real);

    data::set_demo(true);
    assert_ne!(data::carpeta_datos(), dir_real);
    let mut demo = data::cargar();
    assert_eq!(demo.asesores.len(), 14); // la demo no ve nada de lo real
    demo.negocios.clear();
    data::guardar(&demo).unwrap();
    let tmp2 = format!("{}_2", tmp);
    data::exportar(&tmp2).unwrap();
    data::vaciar().unwrap();
    data::importar(&tmp2).unwrap();
    data::restaurar_demo().unwrap();

    data::set_demo(false);
    assert_eq!(foto(&dir_real), antes, "la demo modifico la carpeta del modo Real");
    assert_eq!(data::cargar().asesores.len(), 1);
    // los respaldos automaticos hechos desde la demo quedan marcados como demo
    let raiz = std::path::Path::new(&dir_real).parent().unwrap().join("respaldos");
    for e in std::fs::read_dir(raiz).unwrap() {
        assert!(e.unwrap().file_name().to_string_lossy().starts_with("demo_"));
    }
}

#[test]
fn t_fechas() {
    assert_eq!(normalizar_fecha("17/04/2026").as_deref(), Some("2026-04-17"));
    assert_eq!(normalizar_fecha("2026-02-30"), None);
    assert_eq!(dia_semana("2026-09-23"), Some("Miércoles")); // hoy
    assert_eq!(periodo().len(), 18);
    assert_eq!(periodo()[17], "2027-09");
    assert_eq!(dinero(385115.0), "$385,115.00");
    assert_eq!(dinero(-0.001), "$0.00");
}
