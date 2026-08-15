import { invoke } from "@tauri-apps/api/core";

document.getElementById("btn-ping").addEventListener("click", async () => {
  const respuesta = await invoke("ping");
  document.getElementById("resultado").textContent = respuesta;
});
