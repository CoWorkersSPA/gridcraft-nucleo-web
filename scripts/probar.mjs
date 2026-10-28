// Prueba de humo del paquete: lo que la interfaz lee de `Hoja.state()` y `Hoja.viewport()` y los comandos que lo mueven.
//   node scripts/probar.mjs <carpeta del paquete>
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { join, resolve } from "node:path";
import { pathToFileURL } from "node:url";

const carpeta = resolve(process.argv[2] ?? "");
const { default: iniciar, Hoja } = await import(pathToFileURL(join(carpeta, "gridcraft_web_core.js")).href);
await iniciar({ module_or_path: readFileSync(join(carpeta, "gridcraft_web_core_bg.wasm")) });

const h = new Hoja();
const run = (id, p = {}) => JSON.parse(h.run(id, JSON.stringify(p)));
const estado = () => JSON.parse(h.state());
const celdas = (r0, c0, r1, c1) => JSON.parse(h.viewport(r0, c0, r1, c1));
const texto = (r, c) => celdas(r, c, r, c).cells.find(([f, k]) => f === r && k === c)?.[2];

// Libro en blanco: una hoja, el cursor en A1, nada que deshacer.
let s = estado();
assert.ok(s.sheets.length >= 1, "hojas");
assert.equal(s.sel.active, "A1");
assert.equal(s.canUndo, false);
assert.equal(s.dirty, false);

// Escribir valores y fórmulas, y ver el resultado formateado.
run("cell.set", { cell: "A1", input: "2" });
run("cell.set", { cell: "A2", input: "3" });
run("cell.set", { cell: "A3", input: "=SUM(A1:A2)" });
run("cell.set", { cell: "B1", input: "hola" });
assert.equal(texto(2, 0), "5", "la fórmula se calcula");
assert.equal(texto(0, 1), "hola");
const v = celdas(0, 0, 3, 1);
assert.equal(v.cells.find(([r, c]) => r === 0 && c === 0)[4], 2, "tipo número");
assert.equal(v.cells.find(([r, c]) => r === 0 && c === 1)[4], 1, "tipo texto");

// La barra de fórmulas muestra lo que se escribió, no el resultado.
run("selection.set", { cell: "A3" });
s = estado();
assert.equal(s.sel.active, "A3");
assert.equal(s.formula, "=SUM(A1:A2)");
assert.equal(s.dirty, true);
assert.equal(s.canUndo, true);

// Formato: negrita y centrado se leen del estilo de la celda activa.
run("selection.set", { cell: "B1" });
run("home.bold");
run("home.alignCenter");
s = estado();
assert.equal(s.style.b, true, "negrita");
assert.equal(s.style.h, "center");

// Deshacer y rehacer.
run("edit.undo");
assert.equal(estado().style.h, "general");
assert.equal(estado().style.b, true, "la negrita sigue: se deshizo solo el centrado");
run("edit.redo");
assert.equal(estado().style.h, "center");

// Geometría: las columnas tienen ancho y las filas alto, y se puede volver de una distancia a su columna.
assert.ok(h.col_width(0) > 0 && h.row_height(0) > 0);
assert.equal(h.col_at(h.col_left(3) + 1), 3);
assert.equal(h.row_at(h.row_top(7) + 1), 7);
assert.equal(Hoja.a1(2, 1), "B3");

// Guardar como .xlsx y abrirlo de nuevo en otra hoja: los datos y el formato vuelven.
const xlsx = h.save("prueba.xlsx");
assert.ok(xlsx.length > 1000, "xlsx");
const g = new Hoja();
const avisos = JSON.parse(g.open("prueba.xlsx", xlsx));
assert.ok(Array.isArray(avisos));
const celdasG = JSON.parse(g.viewport(0, 0, 3, 1));
assert.equal(celdasG.cells.find(([r, c]) => r === 2 && c === 0)[2], "5");
assert.equal(JSON.parse(g.state()).dirty, false);

// CSV, y un comando que el motor rechaza llega como excepción.
assert.ok(new TextDecoder().decode(h.save("prueba.csv")).includes("hola"));
assert.throws(() => h.run("no.existe", "{}"));
assert.throws(() => h.save("prueba.cvs"));

console.log("gridcraft-nucleo-web: prueba de humo bien");
