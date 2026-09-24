# InmoCore — KAVELA Servicios Inmobiliarios

Sistema de gestión inmobiliaria basado en el Excel `sistema_Kavela_1_1.xlsx`.
Dos interfaces con la MISMA lógica:

- **Slint** (raíz, `src/main.rs` + `ui/app.slint`): app de escritorio. `cargo run`
- **Tauri** (`src-tauri/` + `src/index.html|main.js|styles.css`): `npm install` y `npm run tauri dev`

## Dónde vive cada cosa

| Archivo | Qué hace |
|---|---|
| `models.rs` | Datos que se digitan (una struct por hoja del Excel) |
| `logic.rs` | Las fórmulas del Excel (comisiones, puntaje, rankings, mensualidad, finanzas, CRM, guardia) |
| `modulos.rs` | Listados, formularios y validaciones (compartido por Slint y Tauri) |
| `data.rs` / `seed.rs` | CSV en `<carpeta de datos>/kavela/`; la primera vez se cargan los datos del Excel |

`models.rs`, `data.rs`, `logic.rs`, `modulos.rs` y `seed.rs` están **duplicados** en `src/` y
`src-tauri/src/` (idénticos). Si cambias uno, copia el cambio al otro.

## Modo Real y modo Demo

- **Real**: tus datos verdaderos. Arranca **en blanco** (solo la estructura) y guarda en `<datos>/kavela/`.
- **Demo**: datos de ejemplo del Excel para probar sin miedo; guarda en `<datos>/kavela_demo/`.
- Se cambia con el botón de abajo a la izquierda (el modo se recuerda al cerrar). Nunca se mezclan.
- **Datos y respaldo**: exportar/importar carpetas de CSV, vaciar todo y restaurar la demo. Importar y vaciar
  hacen antes un respaldo automático en `<datos>/respaldos/`.

## Verificación

`cargo test` (en la raíz) comprueba que el programa da las mismas cifras que el Excel
(`src/verificacion.rs`).
