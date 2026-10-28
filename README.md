# gridcraft-nucleo-web

El núcleo de [GridCraft](https://github.com/storytold/gridcraft) para el navegador, **sin interfaz de Rust**: el motor
(`gridcraft-engine`: libro de cálculo, fórmulas, formato, historial, 300 y tantos comandos) detrás de wasm-bindgen, en un solo
wasm. La interfaz es HTML (`bff/apps/gridcraft`, Astro + Svelte); aquí solo está lo que hace falta para construir y publicar el wasm.

A diferencia de [wordcraft-nucleo-web](https://github.com/CoWorkersSPA/wordcraft-nucleo-web) no hay parche: el motor de gridcraft no
depende de la interfaz y compila a wasm tal cual, así que este repo lo usa como dependencia por commit (`UPSTREAM`) y agrega un
crate chico (`crates/web-core`).

## La interfaz del wasm (`Hoja`)

| | |
|---|---|
| `new Hoja()` | un libro en blanco |
| `open(nombre, bytes)` | abre .xlsx, .xlsb, .xls, .ods, .csv o .tsv; devuelve los avisos (JSON) |
| `save(nombre)` | los bytes en el formato de la extensión (.xlsx, .csv, .tsv, .json, .html) |
| `mark_saved()` | da por guardado el libro |
| `run(id, params)` | corre un comando de gridcraft (`cell.set`, `home.bold`, `edit.undo`…); devuelve su resultado (JSON) o lanza |
| `state()` | título, hojas, selección, barra de fórmulas, estilo de la celda activa, deshacer/rehacer (JSON) |
| `viewport(r0, c0, r1, c1)` | las celdas con algo en ese rectángulo, ya formateadas, y sus estilos (JSON) |
| `col_width`, `row_height`, `col_left`, `row_top`, `col_at`, `row_at` | geometría de la hoja activa |

## Construir en local

```bash
rustup target add wasm32-unknown-unknown   # y tener node
scripts/construir.sh 0.12.1-1              # salida/gridcraft-nucleo-web-0.12.1-1.tar.gz y su .sha256
node scripts/probar.mjs trabajo/gridcraft-nucleo-web-0.12.1-1
```

## Publicar una versión

Subir una etiqueta `v<versión de gridcraft>-<n>` (`git tag v0.12.1-1 && git push --tags`): el CI construye, prueba y publica la
release con el `.tar.gz` y su SHA-256. bff fija esa versión y esa suma en `apps/gridcraft/nucleo.json`.
