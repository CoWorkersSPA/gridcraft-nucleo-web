// Escribe MANIFEST.json en el paquete: qué contiene, de dónde sale y la suma SHA-256 de cada archivo.
//   node scripts/manifiesto.mjs <carpeta> <versión> <repo de gridcraft> <commit> <versión de wasm-bindgen>
import { createHash } from "node:crypto";
import { readdirSync, readFileSync, statSync, writeFileSync } from "node:fs";
import { join, relative } from "node:path";

const [carpeta, version, repo, commit, wasmBindgen] = process.argv.slice(2);
if (!carpeta || !version || !repo || !commit) {
  console.error("uso: manifiesto.mjs <carpeta> <versión> <repo> <commit> [wasm-bindgen]");
  process.exit(1);
}
const archivos = [];
(function recorrer(d) {
  for (const n of readdirSync(d).sort()) {
    const ruta = join(d, n);
    if (statSync(ruta).isDirectory()) recorrer(ruta);
    else if (n !== "MANIFEST.json") archivos.push(ruta);
  }
})(carpeta);
const manifiesto = {
  nombre: "gridcraft-nucleo-web",
  version,
  gridcraft: { repo, commit },
  wasm_bindgen: wasmBindgen ?? null,
  archivos: Object.fromEntries(
    archivos.map((a) => [relative(carpeta, a), { bytes: statSync(a).size, sha256: createHash("sha256").update(readFileSync(a)).digest("hex") }]),
  ),
};
writeFileSync(join(carpeta, "MANIFEST.json"), JSON.stringify(manifiesto, null, 2) + "\n");
console.log(`MANIFEST.json: ${archivos.length} archivos`);
