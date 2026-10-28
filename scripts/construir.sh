#!/usr/bin/env bash
# Construye el núcleo web de GridCraft: compila a wasm el crate `gridcraft-web-core` (el motor de gridcraft, que depende de
# gridcraft-engine por el commit de UPSTREAM) y deja en salida/ un .tar.gz con el wasm, su JS y sus tipos, la versión comprimida
# (.br y .gz) del wasm y su suma SHA-256. A diferencia de wordcraft no hay parche ni datos aparte: el motor es de bytes y no pide
# fuentes, y el wasm es uno solo.
#
#   scripts/construir.sh <versión>        p. ej. scripts/construir.sh 0.12.1-1
#
# Necesita: node, rustup con el target wasm32-unknown-unknown, y (se instalan solos con `cargo install` si faltan)
# wasm-bindgen-cli, de la misma versión que fija el Cargo.lock, y wasm-opt.
set -euo pipefail

raiz="$(cd "$(dirname "$0")/.." && pwd)"
# shellcheck source=../UPSTREAM
source "$raiz/UPSTREAM"
version="${1:?uso: scripts/construir.sh <versión>   (p. ej. 0.12.1-1)}"
trabajo="${TRABAJO:-$raiz/trabajo}"
salida="${SALIDA:-$raiz/salida}"
nombre="gridcraft-nucleo-web-${version}"
paquete="$trabajo/$nombre"

suma() { if command -v sha256sum >/dev/null; then sha256sum "$@"; else shasum -a 256 "$@"; fi; }

# El commit de UPSTREAM y el `rev` del Cargo.toml del crate deben ser el mismo.
grep -q "rev = \"$COMMIT\"" "$raiz/crates/web-core/Cargo.toml" || { echo "UPSTREAM ($COMMIT) y crates/web-core/Cargo.toml no coinciden" >&2; exit 1; }

# La versión de wasm-bindgen que fija el Cargo.lock: el CLI y el crate deben coincidir.
wbg="$(awk '/^name = "wasm-bindgen"$/ { getline; gsub(/"/, "", $3); print $3; exit }' "$raiz/Cargo.lock")"
[ -n "$wbg" ] || { echo "no encuentro wasm-bindgen en Cargo.lock (corre cargo generate-lockfile)" >&2; exit 1; }
if ! command -v wasm-bindgen >/dev/null || [ "$(wasm-bindgen --version | awk '{print $2}')" != "$wbg" ]; then
  echo "→ cargo install wasm-bindgen-cli@$wbg"
  cargo install --locked "wasm-bindgen-cli@$wbg"
fi
command -v wasm-opt >/dev/null || { echo "→ cargo install wasm-opt"; cargo install --locked wasm-opt; }

echo "→ cargo build (tamaño: opt-level z, LTO completo, panic abort; SIMD128)"
(
  cd "$raiz"
  export CARGO_PROFILE_RELEASE_OPT_LEVEL=z CARGO_PROFILE_RELEASE_LTO=fat \
    CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1 CARGO_PROFILE_RELEASE_PANIC=abort
  export RUSTFLAGS="${RUSTFLAGS:-} -C target-feature=+simd128"
  cargo build --release --locked --target wasm32-unknown-unknown -p gridcraft-web-core
)

echo "→ wasm-bindgen y wasm-opt"
rm -rf "$paquete"
mkdir -p "$paquete" "$salida"
wasm-bindgen --target web --typescript --out-dir "$paquete" \
  "${CARGO_TARGET_DIR:-$raiz/target}/wasm32-unknown-unknown/release/gridcraft_web_core.wasm"
wasm-opt -Oz --all-features -o "$paquete/o.wasm" "$paquete/gridcraft_web_core_bg.wasm"
mv "$paquete/o.wasm" "$paquete/gridcraft_web_core_bg.wasm"
cp "$raiz/LICENCIA.md" "$paquete/LICENCIA.md"

echo "→ comprimiendo (brotli y gzip junto al wasm, para que bff no lo haga en cada build)"
node "$raiz/scripts/comprimir.mjs" "$paquete"
node "$raiz/scripts/manifiesto.mjs" "$paquete" "$version" "$REPO" "$COMMIT" "$wbg"

echo "→ empaquetando"
tar -czf "$salida/$nombre.tar.gz" -C "$trabajo" "$nombre"
(cd "$salida" && suma "$nombre.tar.gz" > "$nombre.tar.gz.sha256")
cat "$salida/$nombre.tar.gz.sha256"
ls -l "$salida/$nombre.tar.gz" | awk '{print $5, "bytes"}'
