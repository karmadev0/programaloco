import { invoke } from "@tauri-apps/api/core";

const content = document.getElementById("content");
const botones = document.querySelectorAll(".nav-btn");

botones.forEach((btn) => {
  btn.addEventListener("click", () => {
    botones.forEach((b) => b.classList.remove("activo"));
    btn.classList.add("activo");
    render(btn.dataset.view);
  });
});

function money(n) {
  return "$" + Number(n).toLocaleString("es-VE", { maximumFractionDigits: 2 });
}

function pct(n) {
  return (Number(n) * 100).toFixed(1) + "%";
}

function tabla(columnas, filas) {
  const thead = "<tr>" + columnas.map((c) => `<th>${c.label}</th>`).join("") + "</tr>";
  const tbody = filas
    .map(
      (fila) =>
        "<tr>" +
        columnas.map((c) => `<td>${c.render ? c.render(fila) : fila[c.key] ?? ""}</td>`).join("") +
        "</tr>"
    )
    .join("");
  return `<table><thead>${thead}</thead><tbody>${tbody}</tbody></table>`;
}

function badgeMatch(nivel) {
  const clase = nivel === "MATCH ALTO" ? "badge-alto" : nivel === "MATCH MEDIO" ? "badge-medio" : "badge-no";
  return `<span class="badge ${clase}">${nivel}</span>`;
}

function badgeLegal(estatus) {
  const clase = estatus === "LISTO PARA FIRMA" ? "badge-listo" : "badge-incompleto";
  return `<span class="badge ${clase}">${estatus}</span>`;
}

function badgeActividad(nivel) {
  const clase = nivel === "ALTA PRODUCTIVIDAD" ? "badge-alta" : "badge-baja";
  return `<span class="badge ${clase}">${nivel}</span>`;
}

function siNo(v) {
  return v ? "SI" : "NO";
}

// ---------------------------------------------------------------------
// Vistas de solo lectura
// ---------------------------------------------------------------------

async function vistaDashboard() {
  const k = await invoke("get_dashboard");
  content.innerHTML = `
    <h2>Dashboard</h2>
    <div class="kpis">
      <div class="kpi"><div class="valor">${money(k.total_comisiones_oficina)}</div><div class="etiqueta">Comisiones oficina</div></div>
      <div class="kpi"><div class="valor">${k.matches_disponibles}</div><div class="etiqueta">Matches disponibles</div></div>
      <div class="kpi"><div class="valor">${k.expedientes_listos_firma}</div><div class="etiqueta">Expedientes listos para firma</div></div>
      <div class="kpi"><div class="valor">${money(k.pagos_pendientes)}</div><div class="etiqueta">Pagos pendientes</div></div>
    </div>
  `;
}

async function vistaAsesores() {
  const lista = await invoke("get_asesores");
  content.innerHTML =
    "<h2>Ranking de Asesores</h2>" +
    tabla(
      [
        { label: "#", key: "posicion_ranking" },
        { label: "Nombre", key: "nombre" },
        { label: "Ventas", key: "ventas_concretadas" },
        { label: "Alquileres", key: "alquileres_concretados" },
        { label: "Facturado", render: (f) => money(f.total_facturado) },
        { label: "% Asistencia", render: (f) => pct(f.pct_asistencia) },
        { label: "Score", render: (f) => f.score.toFixed(1) },
      ],
      lista
    );
}

async function vistaMatching() {
  const lista = await invoke("get_matching");
  content.innerHTML =
    "<h2>Matching Oferta / Demanda</h2>" +
    tabla(
      [
        { label: "ID", key: "id" },
        { label: "Cliente", key: "cliente_buscador" },
        { label: "Zona", key: "zona_deseada" },
        { label: "Presupuesto", render: (f) => money(f.presupuesto_max) },
        { label: "Inmueble", key: "inmueble_matcheado" },
        { label: "Precio lista", render: (f) => money(f.precio_lista) },
        { label: "Días venc.", key: "dias_hasta_venc" },
        { label: "Match", render: (f) => badgeMatch(f.nivel_match) },
      ],
      lista
    );
}

async function vistaLegal() {
  const lista = await invoke("get_legal");
  content.innerHTML =
    "<h2>Checklist Legal</h2>" +
    tabla(
      [
        { label: "Inmueble", key: "cod_inmueble" },
        { label: "Propietario", key: "propietario" },
        { label: "Título", render: (f) => siNo(f.titulo_propiedad) },
        { label: "Cédula/RIF", render: (f) => siNo(f.cedula_rif) },
        { label: "Catastro", render: (f) => siNo(f.ficha_catastral) },
        { label: "Solvencia", render: (f) => siNo(f.solvencia_municipal) },
        { label: "Contrato", key: "borrador_contrato" },
        { label: "Estatus", render: (f) => badgeLegal(f.estatus_general) },
      ],
      lista
    );
}

async function vistaEmbudo() {
  const lista = await invoke("get_embudo");
  content.innerHTML =
    "<h2>Embudo Operativo</h2>" +
    tabla(
      [
        { label: "Asesor", key: "nombre_asesor" },
        { label: "Llamadas", key: "llamadas_realizadas" },
        { label: "Citas", key: "citas_captacion" },
        { label: "Visitas", key: "visitas_guiadas" },
        { label: "Ofertas", key: "ofertas_recibidas" },
        { label: "Cierres", key: "cierres_mes" },
        { label: "Tasa conv.", render: (f) => pct(f.tasa_conversion) },
        { label: "Actividad", render: (f) => badgeActividad(f.nivel_actividad) },
      ],
      lista
    );
}

async function vistaCaptaciones() {
  const lista = await invoke("get_captaciones");
  content.innerHTML =
    "<h2>Captaciones</h2>" +
    tabla(
      [
        { label: "Inmueble", key: "cod_inmueble" },
        { label: "Tipo", key: "tipo_propiedad" },
        { label: "Captador", key: "id_captador" },
        { label: "Fecha", key: "fecha_captacion" },
        { label: "Precio", render: (f) => money(f.precio_lista) },
        { label: "Estatus", key: "estatus" },
        { label: "Web", render: (f) => siNo(f.publicado_web) },
        { label: "RRSS", render: (f) => siNo(f.publicado_rrss) },
      ],
      lista
    );
}

async function vistaCierres() {
  const lista = await invoke("get_cierres");
  content.innerHTML =
    "<h2>Cierres</h2>" +
    tabla(
      [
        { label: "ID", key: "id" },
        { label: "Fecha", key: "fecha_cierre" },
        { label: "Inmueble", key: "cod_inmueble" },
        { label: "Tipo", key: "tipo_operacion" },
        { label: "Monto", render: (f) => money(f.monto_operacion) },
        { label: "Captador", key: "nombre_captador" },
        { label: "Cerrador", key: "nombre_cerrador" },
        { label: "Com. oficina", render: (f) => money(f.comision_oficina) },
        { label: "Pago asesores", render: (f) => money(f.pago_asesores) },
      ],
      lista
    );
}

async function vistaFinanzas() {
  const r = await invoke("get_finanzas");
  content.innerHTML =
    "<h2>Finanzas</h2>" +
    `<div class="kpis">
      <div class="kpi"><div class="valor">${money(r.total_pagado)}</div><div class="etiqueta">Total pagado</div></div>
      <div class="kpi"><div class="valor">${money(r.total_pendiente)}</div><div class="etiqueta">Total pendiente</div></div>
    </div>` +
    tabla(
      [
        { label: "ID", key: "id" },
        { label: "Fecha", key: "fecha" },
        { label: "Semana", key: "semana" },
        { label: "Flujo", key: "tipo_flujo" },
        { label: "Categoría", key: "categoria" },
        { label: "Monto", render: (f) => money(f.monto) },
        { label: "Estatus", render: (f) => `<span class="badge ${f.estatus_pago === "Pagado" ? "badge-alto" : "badge-pendiente"}">${f.estatus_pago}</span>` },
      ],
      r.items
    );
}

async function vistaReportes() {
  const lista = await invoke("get_reportes");
  content.innerHTML =
    "<h2>Reportes a Propietarios</h2>" +
    tabla(
      [
        { label: "Inmueble", key: "cod_inmueble" },
        { label: "Propietario", key: "propietario" },
        { label: "Teléfono", key: "telefono" },
        { label: "Visitas", key: "visitas_agendadas" },
        { label: "Ofertas", key: "ofertas_recibidas" },
        { label: "Canales", key: "canales_publicacion" },
        { label: "Estatus", key: "estatus_envio" },
      ],
      lista
    );
}

// ---------------------------------------------------------------------
// Formularios de alta — los selects de asesor se llenan solos (por
// nombre, no hay que tipear el ID a mano).
// ---------------------------------------------------------------------

async function opcionesAsesores(seleccionActual) {
  const asesores = await invoke("get_asesores_lista");
  return asesores
    .map((a) => `<option value="${a.id}">${a.nombre} (${a.id})</option>`)
    .join("");
}

function mostrarMensaje(ok, texto) {
  const p = document.createElement("p");
  p.className = ok ? "mensaje-ok" : "mensaje-error";
  p.textContent = texto;
  content.querySelector("form").after(p);
}

async function formCierre() {
  const opciones = await opcionesAsesores();
  content.innerHTML = `
    <h2>Nuevo cierre</h2>
    <form id="f-cierre">
      <label>ID cierre <input name="id" placeholder="CR-004" required /></label>
      <label>Fecha <input name="fecha_cierre" type="date" required /></label>
      <label>Código inmueble <input name="cod_inmueble" placeholder="INM-105" required /></label>
      <label>Tipo operación
        <select name="tipo_operacion">
          <option>Venta</option>
          <option>Alquiler</option>
        </select>
      </label>
      <label>Monto operación ($) <input name="monto_operacion" type="number" step="0.01" required /></label>
      <label>Captador <select name="id_captador">${opciones}</select></label>
      <label>Cerrador <select name="id_cerrador">${opciones}</select></label>
      <label>% Comisión (ej 0.05 = 5%) <input name="pct_comision_total" type="number" step="0.001" required /></label>
      <button type="submit">Guardar cierre</button>
    </form>
  `;
  document.getElementById("f-cierre").addEventListener("submit", async (e) => {
    e.preventDefault();
    const f = new FormData(e.target);
    try {
      await invoke("add_cierre", {
        id: f.get("id"),
        fechaCierre: f.get("fecha_cierre"),
        codInmueble: f.get("cod_inmueble"),
        tipoOperacion: f.get("tipo_operacion"),
        montoOperacion: parseFloat(f.get("monto_operacion")),
        idCaptador: f.get("id_captador"),
        idCerrador: f.get("id_cerrador"),
        pctComisionTotal: parseFloat(f.get("pct_comision_total")),
      });
      mostrarMensaje(true, "Cierre guardado.");
      e.target.reset();
    } catch (err) {
      mostrarMensaje(false, "Error: " + err);
    }
  });
}

async function formCaptacion() {
  const opciones = await opcionesAsesores();
  content.innerHTML = `
    <h2>Nueva captación</h2>
    <form id="f-captacion">
      <label>Código inmueble <input name="cod_inmueble" placeholder="INM-105" required /></label>
      <label>Tipo propiedad <input name="tipo_propiedad" placeholder="Apartamento" required /></label>
      <label>Captador <select name="id_captador">${opciones}</select></label>
      <label>Fecha captación <input name="fecha_captacion" type="date" required /></label>
      <label>Precio lista ($) <input name="precio_lista" type="number" step="0.01" required /></label>
      <label>Estatus
        <select name="estatus">
          <option>Disponible</option>
          <option>Cerrado</option>
        </select>
      </label>
      <label class="checkbox"><input name="publicado_web" type="checkbox" /> Publicado en web</label>
      <label class="checkbox"><input name="publicado_rrss" type="checkbox" /> Publicado en RRSS</label>
      <button type="submit">Guardar captación</button>
    </form>
  `;
  document.getElementById("f-captacion").addEventListener("submit", async (e) => {
    e.preventDefault();
    const f = new FormData(e.target);
    try {
      await invoke("add_captacion", {
        codInmueble: f.get("cod_inmueble"),
        tipoPropiedad: f.get("tipo_propiedad"),
        idCaptador: f.get("id_captador"),
        fechaCaptacion: f.get("fecha_captacion"),
        precioLista: parseFloat(f.get("precio_lista")),
        estatus: f.get("estatus"),
        publicadoWeb: f.get("publicado_web") === "on",
        publicadoRrss: f.get("publicado_rrss") === "on",
      });
      mostrarMensaje(true, "Captación guardada.");
      e.target.reset();
    } catch (err) {
      mostrarMensaje(false, "Error: " + err);
    }
  });
}

async function formFinanza() {
  content.innerHTML = `
    <h2>Nueva transacción financiera</h2>
    <form id="f-finanza">
      <label>ID transacción <input name="id" placeholder="TR-004" required /></label>
      <label>Fecha <input name="fecha" type="date" required /></label>
      <label>Semana <input name="semana" placeholder="Semana 28" required /></label>
      <label>Tipo flujo
        <select name="tipo_flujo">
          <option>Ingreso</option>
          <option>Egreso</option>
        </select>
      </label>
      <label>Categoría / Asesor <input name="categoria" required /></label>
      <label>Monto ($) <input name="monto" type="number" step="0.01" required /></label>
      <label>Estatus pago
        <select name="estatus_pago">
          <option>Pagado</option>
          <option>Pendiente</option>
        </select>
      </label>
      <button type="submit">Guardar transacción</button>
    </form>
  `;
  document.getElementById("f-finanza").addEventListener("submit", async (e) => {
    e.preventDefault();
    const f = new FormData(e.target);
    try {
      await invoke("add_finanza", {
        id: f.get("id"),
        fecha: f.get("fecha"),
        semana: f.get("semana"),
        tipoFlujo: f.get("tipo_flujo"),
        categoria: f.get("categoria"),
        monto: parseFloat(f.get("monto")),
        estatusPago: f.get("estatus_pago"),
      });
      mostrarMensaje(true, "Transacción guardada.");
      e.target.reset();
    } catch (err) {
      mostrarMensaje(false, "Error: " + err);
    }
  });
}

// ---------------------------------------------------------------------

const vistas = {
  dashboard: vistaDashboard,
  asesores: vistaAsesores,
  matching: vistaMatching,
  legal: vistaLegal,
  embudo: vistaEmbudo,
  captaciones: vistaCaptaciones,
  cierres: vistaCierres,
  finanzas: vistaFinanzas,
  reportes: vistaReportes,
  "form-cierre": formCierre,
  "form-captacion": formCaptacion,
  "form-finanza": formFinanza,
};

async function render(nombre) {
  content.innerHTML = '<p class="cargando">Cargando…</p>';
  try {
    await vistas[nombre]();
  } catch (err) {
    content.innerHTML = `<p class="mensaje-error">Error cargando la vista: ${err}</p>`;
  }
}

// Arranca en el dashboard
document.querySelector('[data-view="dashboard"]').classList.add("activo");
render("dashboard");
