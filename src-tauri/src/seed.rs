// seed.rs — datos iniciales tomados de sistema_Kavela_1_1.xlsx.
// Se escriben a disco SOLO la primera vez (si no existe aun la carpeta de datos).

pub const ASESORES: &str = r#"codigo,nombre_completo,nombre_corto,cedula,cargo,formato,fecha_ingreso,telefono
AS-001,CARLOS ENRIQUE CACERES RODRIGUEZ,Carlos Caceres,14503324,Broker/Abg,,,
AS-002,DAYANA KARINA RODRIGUEZ DE CACERES,Karina Rodriguez,16409015,Broker/Abg,,,
AS-003,ESTEFANNY DE LOS ANGELES CARDENAS GARCIA,Estefanny Cardenas,24778903,Agente Inmobiliario,0.5,2026-04-21,
AS-004,GABRIELA CAROLINA RAMIREZ NAVAS,Gabriela Ramirez,19677556,Agente Inmobiliario,0.5,2026-08-17,
AS-005,GLEYDIS LOUISHANNA ARDILA PARRA,Louishanna Ardila,27361442,Agente Inmobiliario,0.55,2026-04-28,
AS-006,JOSNEIDA RAMIREZ,Josneida Ramirez,20999399,Administrativo,,2026-04-25,
AS-007,LUISANA EMPERATRIZ PAEZ ORTIZ,Luisana Paez,33456256,Agente Inmobiliario,0.5,2026-08-18,
AS-008,MARY REDONDO MENDEZ,Mary Redondo,29507926,Agente Inmobiliario,0.5,2026-04-21,
AS-009,MARYLENA CARREÑO MANJARREZ,Marylena Carreño,13709320,Agente Inmobiliario,0.5,2026-04-21,
AS-010,SORELYS EULALIA BRITO SALGADO,Sorelys Salgado,15550662,Agente Inmobiliario,0.5,2026-04-21,
AS-011,VALENTINA PARADISO GHOLLET,Valentina Paradiso,25967203,Agente Inmobiliario,0.5,2026-04-21,
AS-012,YANETH CAROLINA RODRIGUEZ RUIZ,Yaneth Rodriguez,16981094,Agente Inmobiliario,0.5,2026-06-02,
AS-013,YENNIFER SHIRLEY BELTRAN DAZA,Yennifer Beltran,26493054,Agente Inmobiliario,0.5,2026-04-21,
AS-014,YULE DEL CARMEN CARDENAS GARCIA,Yule Cardenas,23547026,Agente Inmobiliario,0.5,2026-04-21,
"#;

pub const NEGOCIOS: &str = r#"id,fecha,asesor,formato,punta_captadora,punta_compradora,punta_arrendataria,asesor_otra_punta,monto,tipo_inmueble,ubicacion,canon,tipo_negocio,fin,comision_total,origen,notas
1,2026-04-17,Karina Rodriguez,0.5,KARINA RODRIGUEZ KAVELA,YEDZY GOMEZ REMAX PLATINIUM,,,64000,Finca,San Isidro Municipio Fernan. Feo,,Venta,Agro,1600,Histórico,
2,2026-04-21,Karina Rodriguez,0.5,KARLA RONDERO CONECTA,KARINA RODRIGUEZ KAVELA,,,28000,Parcela,La Estancia Del Norte La Castellana,,Venta,Residencial,700,Histórico,
3,2026-04-24,Karina Rodriguez,0.5,YASMIN LABRADOR INNOVA,KARINA RODRIGUEZ KAVELA,,,52500,Apartamento,Quinimari Torre 44,,Venta,Residencial,1300,Histórico,
4,2026-04-27,Karina Rodriguez,0.5,KARINA RODRIGUEZ KAVELA,,ERIKA PEREZ INDEPENDIENTE,,220,Apartamento,Pirineos,220,Alquiler,Residencial,220,Histórico,
5,2026-04-30,Valentina Paradiso,0.5,VALENTINA PARADISO KAVELA,KRISLEY SALAS INDEPENDIENTE,,,67000,Casa,Barrio Bolivar Calle El Alto,,Venta,Residencial,1675,Histórico,
6,2026-05-07,Karina Rodriguez,0.5,KARINA RODRIGUEZ KAVELA,YELANY SIFONTES LA HOUSE,,,40000,Parcela,Puerto Madero Pueblo Nuevo,,Venta,Residencial,1000,Histórico,
7,2026-05-16,Karina Rodriguez,0.5,KARINA RODRIGUEZ KAVELA,,ESTEFANNY CARDENAS,,180,Apartamento,Av. Principal De Pueblo Nuevo,180,Alquiler,Residencial,180,Histórico,
8,2026-05-18,Estefanny Cardenas,0.5,KARINA RODRIGUEZ KAVELA,,ESTEFANNY CARDENAS,,180,Apartamento,Av. Principal De Pueblo Nuevo,180,Alquiler,Residencial,180,Histórico,
9,2026-05-26,Yule Cardenas,0.5,LILIANA VALERO DE CONECTA,YULE CARDENAS DE KAVELA,,,49000,Apartamento,Kioskos Residencia Santa Rita,,Venta,Residencial,1225,Histórico,
10,2026-06-08,Karina Rodriguez,0.5,KARINA RODRIGUEZ KAVELA,,JUAN CARLOS NIÑO,,290,Apartamento,Av. Principal De Pueblo Nuevo,290,Alquiler,Residencial,290,Histórico,
11,2026-06-26,Karina Rodriguez,0.5,KARINA RODRIGUEZ KAVELA,,LIZIA GOMEZ INMOBLANC,,200,Apartamento,Av. Principal De Pueblo Nuevo,200,Alquiler,Residencial,200,Histórico,
12,2026-06-29,Mary Redondo,0.5,VICTOR MENDOZA REMAX NOBEL,,MARY REDONDO,,450,Apartamento,Av. Ferrero Tamayo,450,Alquiler,Residencial,450,Histórico,
13,2026-07-06,Yule Cardenas,0.5,HUMBERTO REMAX PLATINIUM,,YULE CARDENAS,,300,Apartamento,San Ignacio Ferrero Tamayo,300,Alquiler,Residencial,300,Histórico,
14,2026-07-27,Sorelys Salgado,0.5,JUAN CARLOS REMAX FUTURO,SORELYS SALGADO,,,40000,Apartamento,Los Teques Etapa 17,,Venta,Residencial,1000,Histórico,
15,2026-08-03,Sorelys Salgado,0.5,BELSY DE PINTO,,SORELYS SALGADO,,165,Apartamento,Barrio Libertador,165,Alquiler,Residencial,164,Histórico,
16,2026-08-10,Karina Rodriguez,0.5,KARINA RODRIGUEZ,,MARIANA NUÑEZ REMAX DIAMANTE,,40000,Parcela,Pueblo Nuevo,,Venta,Residencial,500,Histórico,
17,2026-08-12,Valentina Paradiso,0.5,VALENTINA PARADISO,,YENNIFER BELTRAN,,430,Casa,Barrio Bolivar,430,Alquiler,Residencial,430,Histórico,
18,2026-08-12,Yennifer Beltran,0.5,VALENTINA PARADISO,,YENNIFER BELTRAN,,430,Casa,Barrio Bolivar,430,Alquiler,Residencial,430,Histórico,
19,2026-08-15,Gabriela Ramirez,0.5,GLADYS SEPULVEDA REMAX NEW HOME,,GABRIELA RAMIREZ,,730,Casa,Urb. California Suites,730,Alquiler,Residencial,730,Histórico,
20,2026-08-17,Louishanna Ardila,0.55,HUMBERTO MARQUEZ REMAX PLATINIUM,,LOUISHANNA ARDILA,,450,Apartamento,Paramillo Suites,450,Alquiler,Residencial,450,Histórico,
21,2026-08-18,Yennifer Beltran,0.5,EMPORIO CENTRO INMOBILIARIO,,YENNIFER BELTRAN,,350,Casa,Palo Gordo,350,Alquiler,Residencial,350,Histórico,
22,2026-08-21,Louishanna Ardila,0.55,REMAX NEW HOME LEWIS REYES,,LOUISHANNA ARDILA,,240,Apartamento,Santa Teresa,250,Alquiler,Residencial,250,Histórico,
"#;

pub const MENSUALIDAD: &str = r#"codigo,mes,dia
AS-003,2026-04,5
AS-003,2026-05,5
AS-003,2026-06,5
AS-003,2026-07,5
AS-003,2026-08,5
AS-004,2026-04,5
AS-004,2026-05,5
AS-004,2026-06,5
AS-004,2026-07,5
AS-004,2026-08,5
AS-005,2026-04,5
AS-005,2026-05,5
AS-005,2026-06,5
AS-005,2026-07,5
AS-005,2026-08,5
AS-007,2026-04,5
AS-007,2026-05,5
AS-007,2026-06,5
AS-007,2026-07,5
AS-007,2026-08,5
AS-008,2026-04,5
AS-008,2026-05,5
AS-008,2026-06,5
AS-008,2026-07,5
AS-008,2026-08,5
AS-009,2026-04,5
AS-009,2026-05,5
AS-009,2026-06,5
AS-009,2026-07,5
AS-009,2026-08,5
AS-010,2026-04,5
AS-010,2026-05,5
AS-010,2026-06,5
AS-010,2026-07,5
AS-010,2026-08,5
AS-011,2026-04,5
AS-011,2026-05,5
AS-011,2026-06,5
AS-011,2026-07,5
AS-011,2026-08,5
AS-012,2026-04,5
AS-012,2026-05,5
AS-012,2026-06,5
AS-012,2026-07,5
AS-012,2026-08,5
AS-013,2026-04,5
AS-013,2026-05,5
AS-013,2026-06,5
AS-013,2026-07,5
AS-013,2026-08,5
AS-014,2026-04,5
AS-014,2026-05,5
AS-014,2026-06,5
AS-014,2026-07,5
AS-014,2026-08,5
"#;

pub const EGRESOS: &str = r#"mes,arriendo,servicios,marketing,nomina
2026-04,440,25,40,160
2026-05,440,25,50,160
2026-06,440,25,30,160
2026-07,440,25,40,160
2026-08,440,25,50,160
2026-09,0,0,0,0
2026-10,0,0,0,0
2026-11,0,0,0,0
2026-12,0,0,0,0
2027-01,0,0,0,0
2027-02,0,0,0,0
2027-03,0,0,0,0
2027-04,0,0,0,0
2027-05,0,0,0,0
2027-06,0,0,0,0
2027-07,0,0,0,0
2027-08,0,0,0,0
2027-09,0,0,0,0
"#;

pub const INVENTARIO: &str = r#"id,fecha,asesor,tipo_inmueble,ubicacion,precio_venta,canon,tipo_operacion,estado,propietario,telefono,habitaciones,banos,m2,caracteristicas,notas
"#;

pub const CLIENTES: &str = r#"id,fecha_contacto,nombre,telefono,email,zona,tipo_inmueble,presupuesto,tipo_negocio,asesor,fuente,etapa,proximo_contacto,notas,inmueble_sugerido,resultado,fecha_cierre
"#;

pub const GUARDIA: &str = r#"dia,abogado,cedula_colegio,telefono,correo,notas
Lunes,Karina Rodríguez,16409015 / INCES,—,—,
Martes,David Colina,—,—,—,Completar datos
Miércoles,Carlos Cáceres,14503324 / Abg. CJ,—,—,
Jueves,Esposa Colina,—,—,—,Completar datos
Viernes,Karina Rodríguez,16409015 / INCES,—,—,
"#;

