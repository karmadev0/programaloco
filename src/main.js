import { invoke } from "@tauri-apps/api/core";

// Modulos (mismos numeros que modulos.rs): 0 Dashboard, 1 Negocios, 2 Asesores,
// 3 Mensualidad, 4 Finanzas, 5 Inventario, 6 CRM, 7 Guardia, 8 TOP Mensual,
// 9 Rankings, 10 Configuracion, 11 Graficos, 12 Ayuda.
const M_GUARDIA = 7;

const content = document.getElementById("content");
const botones = document.querySelectorAll(".nav-btn");

botones.forEach((btn) => {
  btn.addEventListener("click", () => abrir(Number(btn.dataset.m)));
});

function marcarActivo(m) {
  botones.forEach((b) => b.classList.toggle("activo", Number(b.dataset.m) === m));
}

function esc(v) {
  return String(v ?? "")
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;");
}

function tabla(headers, filas, acciones) {
  const thead =
    "<tr>" + headers.map((h) => `<th>${esc(h)}</th>`).join("") + (acciones ? "<th></th>" : "") + "</tr>";
  const tbody = filas
    .map(
      (f, i) =>
        "<tr>" +
        f.map((c) => `<td>${esc(c)}</td>`).join("") +
        (acciones ? `<td class="acciones">${acciones(i)}</td>` : "") +
        "</tr>"
    )
    .join("");
  return `<div class="tabla-scroll"><table><thead>${thead}</thead><tbody>${tbody}</tbody></table></div>`;
}

function mensajeHtml(msg, error) {
  if (!msg) return "";
  return `<p class="${error ? "mensaje-error" : "mensaje-ok"}">${esc(msg)}</p>`;
}

async function abrir(m, mensaje) {
  marcarActivo(m);
  content.innerHTML = '<p class="cargando">Cargando…</p>';
  try {
    if (m === 0) await vistaDashboard();
    else if (m === 11) await vistaGraficos();
    else if (m === 12) vistaAyuda();
    else await vistaGenerica(m, mensaje);
  } catch (err) {
    content.innerHTML = `<p class="mensaje-error">Error cargando la vista: ${esc(err)}</p>`;
  }
}

// ---------------------------------------------------------------------
// Dashboard
// ---------------------------------------------------------------------

async function vistaDashboard() {
  const d = await invoke("get_dashboard");
  const kpi = (k) =>
    `<div class="kpi"><div class="valor">${esc(k.valor)}</div><div class="etiqueta">${esc(k.etiqueta)}</div></div>`;
  content.innerHTML = `
    <h2>Tablero ejecutivo</h2>
    <div class="kpis">${d.kpis.map(kpi).join("")}</div>
    <h3>Cuadro de honor — período completo</h3>
    <div class="kpis">${d.honor.map(kpi).join("")}</div>
    <h3>Desempeño acumulado por asesor</h3>
    ${tabla(d.headers, d.filas)}
  `;
}

// ---------------------------------------------------------------------
// Pantalla generica (Negocios, Asesores, Mensualidad, Finanzas, Inventario,
// CRM, Guardia, TOP, Rankings, Configuracion)
// ---------------------------------------------------------------------

async function vistaGenerica(m, mensaje) {
  const v = await invoke("get_vista", { modulo: m });
  const acciones =
    v.puede_editar || v.puede_eliminar
      ? (i) =>
          (v.puede_editar ? `<button class="btn-mini" data-editar="${i}">Editar</button>` : "") +
          (v.puede_eliminar ? `<button class="btn-mini btn-peligro" data-eliminar="${i}">Eliminar</button>` : "")
      : null;

  content.innerHTML = `
    <div class="encabezado">
      <h2>${esc(v.titulo)}</h2>
      ${v.puede_crear ? '<button class="btn" id="btn-nuevo">Nuevo</button>' : ""}
    </div>
    ${v.ayuda ? `<p class="ayuda">${esc(v.ayuda)}</p>` : ""}
    <div id="msg">${mensajeHtml(mensaje, false)}</div>
    ${
      m === M_GUARDIA
        ? `<div class="buscador">
             <label>Fecha a consultar <input id="fecha-guardia" placeholder="AAAA-MM-DD" /></label>
             <button class="btn" id="btn-guardia">Consultar</button>
             <strong id="res-guardia"></strong>
           </div>`
        : ""
    }
    ${tabla(v.headers, v.filas, acciones)}
  `;

  const nuevo = document.getElementById("btn-nuevo");
  if (nuevo) nuevo.addEventListener("click", () => formulario(m, null));

  content.querySelectorAll("[data-editar]").forEach((b) =>
    b.addEventListener("click", () => formulario(m, Number(b.dataset.editar)))
  );

  content.querySelectorAll("[data-eliminar]").forEach((b) =>
    b.addEventListener("click", async () => {
      if (b.dataset.armado !== "1") {
        // primer toque arma, segundo confirma
        b.dataset.armado = "1";
        b.textContent = "¿Seguro?";
        return;
      }
      try {
        await invoke("eliminar_registro", { modulo: m, idx: Number(b.dataset.eliminar) });
        abrir(m, "Registro eliminado.");
      } catch (err) {
        document.getElementById("msg").innerHTML = mensajeHtml(err, true);
      }
    })
  );

  const bg = document.getElementById("btn-guardia");
  if (bg) {
    const consultar = async () => {
      const r = await invoke("consultar_guardia", { fecha: document.getElementById("fecha-guardia").value });
      document.getElementById("res-guardia").textContent = r;
    };
    bg.addEventListener("click", consultar);
    document.getElementById("fecha-guardia").addEventListener("keydown", (e) => {
      if (e.key === "Enter") consultar();
    });
  }
}

async function formulario(m, idx) {
  const campos = await invoke("get_campos", { modulo: m, idx });
  const titulo = idx === null ? "Nuevo registro" : "Editar registro";
  content.innerHTML = `
    <h2>${titulo}</h2>
    <div id="msg"></div>
    <form id="f">
      ${campos
        .map((c, i) =>
          c.opciones.length
            ? `<label>${esc(c.etiqueta)}
                 <select name="c${i}">${c.opciones
                   .map((o) => `<option value="${esc(o)}" ${o === c.valor ? "selected" : ""}>${esc(o)}</option>`)
                   .join("")}</select>
               </label>`
            : `<label>${esc(c.etiqueta)}<input name="c${i}" value="${esc(c.valor)}" /></label>`
        )
        .join("")}
      <div class="fila-botones">
        <button type="submit">${idx === null ? "Guardar" : "Actualizar"}</button>
        <button type="button" class="btn-secundario" id="cancelar">Cancelar</button>
      </div>
    </form>
  `;
  document.getElementById("cancelar").addEventListener("click", () => abrir(m));
  document.getElementById("f").addEventListener("submit", async (e) => {
    e.preventDefault();
    const valores = campos.map((_, i) => e.target.elements[`c${i}`].value);
    try {
      const msg = await invoke("guardar_registro", { modulo: m, idx, valores });
      abrir(m, msg);
    } catch (err) {
      // el formulario queda abierto con lo escrito, para corregir
      document.getElementById("msg").innerHTML = mensajeHtml(err, true);
    }
  });
}

// ---------------------------------------------------------------------
// Graficos y ayuda
// ---------------------------------------------------------------------

async function vistaGraficos() {
  const gs = await invoke("get_graficos");
  content.innerHTML =
    "<h2>Gráficos y estadísticas</h2>" +
    gs
      .map(
        (g) => `
      <div class="grafico">
        <h3>${esc(g.titulo)}</h3>
        ${
          g.barras.length
            ? g.barras
                .map(
                  (b) => `<div class="barra">
                    <span class="barra-etiqueta">${esc(b.etiqueta)}</span>
                    <span class="barra-pista"><span class="barra-relleno" style="width:${(b.proporcion * 100).toFixed(1)}%"></span></span>
                    <span class="barra-texto">${esc(b.texto)}</span>
                  </div>`
                )
                .join("")
            : '<p class="cargando">Sin datos todavía.</p>'
        }
      </div>`
      )
      .join("");
}

function vistaAyuda() {
  const tarjeta = (t, p) => `<div class="tarjeta"><h3>${t}</h3><p>${p}</p></div>`;
  content.innerHTML =
    "<h2>Cómo funciona</h2>" +
    tarjeta(
      "Regla general",
      "Los negocios se cargan SOLO en Registro Negocios. Dashboard, Asesores, TOP Mensual, Rankings, Finanzas y Gráficos se calculan solos a partir de ahí."
    ) +
    tarjeta(
      "Registro Negocios",
      "Cada negocio cerrado: fecha, asesor, formato % (lo que se lleva el asesor), puntas, monto, tipo y comisión total. Comisión del asesor = comisión total × formato %; comisión de la oficina = el resto."
    ) +
    tarjeta(
      "Asesores y puntaje",
      "Puntaje = volumen × 0.5 + negocios × 200 + comisión del asesor × 0.3 + captaciones × 100. Una captación es un negocio donde el asesor aparece en 'Punta captadora'."
    ) +
    tarjeta(
      "Mensualidad",
      "Cuota de $10 por agente. Día de pago ≤ 5: a tiempo · ≤ 15: con mora · más de 15: mora severa · sin día: pendiente. Use Editar para poner el día de pago."
    ) +
    tarjeta(
      "Finanzas",
      "Ingresos = comisiones + mensualidades (automático). Los egresos (arriendo, servicios, marketing, nómina) se cargan con Editar en cada mes. Utilidad = ingresos − egresos."
    ) +
    tarjeta(
      "Inventario, CRM y Guardia",
      "Inventario: propiedades en cartera. CRM: seguimiento de clientes (el % de cierre sale de la etapa). Guardia: abogado por día y buscador por fecha."
    );
}

// Pantalla inicial
abrir(0);
