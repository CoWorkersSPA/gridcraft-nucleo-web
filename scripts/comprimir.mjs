// Deja junto a cada wasm y dato del paquete su versión comprimida (<archivo>.br con brotli al máximo y <archivo>.gz con gzip 9), para
// que quien lo consume (bff) no tenga que comprimirlos en cada build: MANIFEST.json trae las sumas del original y de cada comprimido
// y los entrega tal cual. Es determinista (mismos bytes de entrada, mismos bytes de salida).
//   node scripts/comprimir.mjs <carpeta del paquete>      (antes de scripts/manifiesto.mjs)
import { readdirSync, readFileSync, statSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import zlib from "node:zlib";

const carpeta = process.argv[2];
if (!carpeta) {
  console.error("uso: comprimir.mjs <carpeta del paquete>");
  process.exit(1);
}
// Lo que bff sirve comprimido: el wasm.
const COMPRIMIBLE = /\.wasm$/;
let total = 0;
let origen = 0;
let brotli = 0;
(function recorrer(d) {
  for (const n of readdirSync(d).sort()) {
    const ruta = join(d, n);
    if (statSync(ruta).isDirectory()) recorrer(ruta);
    else if (COMPRIMIBLE.test(n)) {
      const datos = readFileSync(ruta);
      const br = zlib.brotliCompressSync(datos, {
        params: { [zlib.constants.BROTLI_PARAM_QUALITY]: 11, [zlib.constants.BROTLI_PARAM_LGWIN]: 24, [zlib.constants.BROTLI_PARAM_SIZE_HINT]: datos.length },
      });
      writeFileSync(`${ruta}.br`, br);
      writeFileSync(`${ruta}.gz`, zlib.gzipSync(datos, { level: 9 }));
      total++;
      origen += datos.length;
      brotli += br.length;
    }
  }
})(carpeta);
console.log(`comprimidos ${total} archivos: ${(origen / 1e6).toFixed(2)} MB → brotli ${(brotli / 1e6).toFixed(2)} MB`);
