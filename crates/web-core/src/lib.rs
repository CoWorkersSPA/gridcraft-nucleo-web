//! GridCraft para el navegador: el motor (`gridcraft-engine`) detrás de wasm-bindgen, sin interfaz de Rust.
//!
//! La interfaz es HTML (bff/apps/gridcraft): dibuja la cuadrícula con lo que entrega `viewport` y manda cada acción como un comando
//! (`run`), igual que la consola, la CLI y el servidor MCP de gridcraft. Aquí no hay estado propio: todo vive en la `Session`.
#![forbid(unsafe_code)]

use gridcraft_engine::core::{CellRef, RangeRef, Value};
use gridcraft_engine::model::{BorderLine, BorderStyle, Color, HAlign, StyleId, VAlign, Visibility};
use gridcraft_engine::{DocState, Session, Trust, display, io};
use serde_json::{Value as Json, json};
use wasm_bindgen::prelude::*;

/// Un libro de cálculo abierto (o en blanco) y su historial.
#[wasm_bindgen]
pub struct Hoja {
    sesion: Session,
}

fn error(e: impl ToString) -> JsError {
    JsError::new(&e.to_string())
}

fn hex(c: &Color, wb: &gridcraft_engine::model::Workbook) -> Option<String> {
    c.hex(&wb.theme)
}

fn borde(l: &BorderLine, wb: &gridcraft_engine::model::Workbook) -> Json {
    if l.style == BorderStyle::None {
        return Json::Null;
    }
    let grosor = match l.style {
        BorderStyle::Medium | BorderStyle::MediumDashed | BorderStyle::MediumDashDot | BorderStyle::MediumDashDotDot => 2,
        BorderStyle::Thick | BorderStyle::Double => 3,
        _ => 1,
    };
    json!([grosor, hex(&l.color, wb).unwrap_or_else(|| "#000000".into())])
}

fn estilo(wb: &gridcraft_engine::model::Workbook, id: StyleId) -> Json {
    let s = wb.styles.get(id);
    let h = match s.align.h {
        HAlign::Left | HAlign::Fill | HAlign::Justify | HAlign::Distributed => "left",
        HAlign::Center | HAlign::CenterAcross => "center",
        HAlign::Right => "right",
        HAlign::General => "general",
    };
    let v = match s.align.v {
        VAlign::Top => "top",
        VAlign::Center => "center",
        _ => "bottom",
    };
    json!({
        "font": s.font.name,
        "size": s.font.size,
        "b": s.font.bold,
        "i": s.font.italic,
        "u": s.font.underline != gridcraft_engine::model::Underline::None,
        "s": s.font.strike,
        "color": hex(&s.font.color, wb),
        "fill": if s.fill.pattern == gridcraft_engine::model::PatternType::Solid { hex(&s.fill.fg, wb) } else { None },
        "h": h,
        "v": v,
        "wrap": s.align.wrap,
        "indent": s.align.indent,
        "nf": s.num_fmt.as_str(),
        "bd": [borde(&s.borders.left, wb), borde(&s.borders.right, wb), borde(&s.borders.top, wb), borde(&s.borders.bottom, wb)],
    })
}

#[wasm_bindgen]
impl Hoja {
    /// Un libro en blanco (con las hojas que pide la configuración por omisión).
    #[wasm_bindgen(constructor)]
    pub fn new() -> Hoja {
        let mut sesion = Session::new();
        sesion.new_workbook();
        Hoja { sesion }
    }

    /// Abre un archivo (.xlsx, .xlsb, .xls, .ods, .csv, .tsv: se reconoce por su contenido) en lugar del libro actual. Devuelve los
    /// avisos de lo que no se pudo conservar, como un arreglo JSON de textos.
    pub fn open(&mut self, nombre: &str, bytes: &[u8]) -> Result<String, JsError> {
        let (wb, avisos) = io::open_bytes(nombre, bytes).map_err(error)?;
        let mut sesion = Session::new();
        sesion.add_document(DocState::new(wb, Some(nombre.to_string()), nombre.to_string(), Trust::FromFile));
        self.sesion = sesion;
        Ok(json!(avisos).to_string())
    }

    /// El libro en el formato que dice la extensión de `nombre` (.xlsx, .csv, .tsv, .json, .html).
    pub fn save(&self, nombre: &str) -> Result<Vec<u8>, JsError> {
        let d = self.sesion.doc().map_err(error)?;
        io::save_bytes(&d.wb, nombre).map_err(error)
    }

    /// Da por guardado el libro: deja de contar como modificado.
    pub fn mark_saved(&mut self) {
        if let Ok(d) = self.sesion.doc_mut() {
            d.saved = d.wb.clone();
        }
    }

    /// Corre un comando (`cell.set`, `home.bold`, `edit.undo`…; los 300 y tantos de gridcraft) y devuelve su resultado como JSON.
    /// Un comando que el motor rechaza llega como excepción con su explicación.
    pub fn run(&mut self, id: &str, params: &str) -> Result<String, JsError> {
        let params: Json = if params.is_empty() { json!({}) } else { serde_json::from_str(params).map_err(error)? };
        let r = self.sesion.execute(id, params).map_err(error)?;
        Ok(r.to_string())
    }

    /// Lo que la interfaz lee tras cada acción: título, hojas, selección, texto de la barra de fórmulas y qué se puede deshacer.
    pub fn state(&self) -> String {
        let Ok(d) = self.sesion.doc() else { return "null".into() };
        let wb = &d.wb;
        let hoja = wb.active();
        let sel = &d.selection;
        let editar = hoja.map(|h| display::edit_text(wb, h, sel.active)).unwrap_or_default();
        let usado = hoja.and_then(|h| h.used_range()).map(|r| json!([r.end.row, r.end.col]));
        json!({
            "title": d.title,
            "dirty": d.is_dirty(),
            "canUndo": !d.undo.is_empty(),
            "canRedo": !d.redo.is_empty(),
            "sheets": wb.sheets.iter().map(|s| json!({"name": s.name, "hidden": s.visibility != Visibility::Visible})).collect::<Vec<_>>(),
            "active": wb.active_sheet,
            "sel": {
                "active": sel.active.a1(),
                "row": sel.active.row,
                "col": sel.active.col,
                "ranges": sel.ranges.iter().map(|r| json!([r.start.row, r.start.col, r.end.row, r.end.col])).collect::<Vec<_>>(),
                "text": sel.current().a1(),
            },
            "formula": editar,
            "used": usado,
            "freeze": hoja.and_then(|h| h.freeze),
            "defaultColW": hoja.map(|h| h.default_col_width),
            "defaultRowH": hoja.map(|h| h.default_row_height),
            "merges": hoja.map(|h| h.merges.iter().map(|r| json!([r.start.row, r.start.col, r.end.row, r.end.col])).collect::<Vec<_>>()),
            "style": hoja.map(|h| estilo(wb, h.style_id(sel.active))),
        })
        .to_string()
    }

    /// Las celdas con algo (texto, formato) en un rectángulo de la hoja activa, con su texto ya formateado, más los estilos que usan:
    /// `{"cells": [[fila, col, texto, estilo, tipo]], "styles": {"<id>": {…}}}`. El tipo: 1 texto, 2 número, 3 lógico, 4 error, 0 vacía.
    pub fn viewport(&self, r0: u32, c0: u32, r1: u32, c1: u32) -> String {
        let Ok(d) = self.sesion.doc() else { return "null".into() };
        let wb = &d.wb;
        let Some(hoja) = wb.active() else { return "null".into() };
        let mut celdas = Vec::new();
        let mut estilos = serde_json::Map::new();
        for r in r0..=r1 {
            for c in c0..=c1 {
                let cr = CellRef::new(r, c);
                let id = hoja.style_id(cr);
                let valor = hoja.value(cr);
                let tipo = match valor {
                    Value::Empty => 0,
                    Value::Text(_) => 1,
                    Value::Number(_) => 2,
                    Value::Bool(_) => 3,
                    Value::Error(_) => 4,
                    Value::Array(_) => 1,
                };
                if tipo == 0 && id == StyleId::default() {
                    continue;
                }
                let texto = if tipo == 0 { String::new() } else { display::cell_text(wb, hoja, cr) };
                estilos.entry(id.0.to_string()).or_insert_with(|| estilo(wb, id));
                celdas.push(json!([r, c, texto, id.0, tipo]));
            }
        }
        json!({"cells": celdas, "styles": estilos}).to_string()
    }

    /// El ancho de la columna y el alto de la fila (0 si están ocultas).
    pub fn col_width(&self, col: u32) -> f32 {
        self.sesion.doc().ok().and_then(|d| d.wb.active().map(|h| h.col_width(col))).unwrap_or(0.0)
    }
    pub fn row_height(&self, fila: u32) -> f32 {
        self.sesion.doc().ok().and_then(|d| d.wb.active().map(|h| h.row_height(fila))).unwrap_or(0.0)
    }
    /// Dónde empieza la columna o la fila, contando desde el borde de la hoja.
    pub fn col_left(&self, col: u32) -> f64 {
        self.sesion.doc().ok().and_then(|d| d.wb.active().map(|h| h.col_left(col))).unwrap_or(0.0)
    }
    pub fn row_top(&self, fila: u32) -> f64 {
        self.sesion.doc().ok().and_then(|d| d.wb.active().map(|h| h.row_top(fila))).unwrap_or(0.0)
    }
    /// La columna o la fila que está en esa distancia del borde de la hoja.
    pub fn col_at(&self, x: f64) -> u32 {
        self.sesion.doc().ok().and_then(|d| d.wb.active().map(|h| h.col_at(x))).unwrap_or(0)
    }
    pub fn row_at(&self, y: f64) -> u32 {
        self.sesion.doc().ok().and_then(|d| d.wb.active().map(|h| h.row_at(y))).unwrap_or(0)
    }

    /// Cuántas filas y columnas tiene la hoja como máximo (para el tamaño de la barra de desplazamiento).
    pub fn limits(&self) -> String {
        json!({"rows": gridcraft_engine::core::MAX_ROWS, "cols": gridcraft_engine::core::MAX_COLS}).to_string()
    }

    /// Los comandos que acepta `run`, con su etiqueta, atajo, parámetros y si están habilitados ahora (arreglo JSON).
    pub fn commands(&self) -> String {
        serde_json::to_string(&self.sesion.commands()).unwrap_or_else(|_| "[]".into())
    }

    /// La dirección A1 de una celda y el rango A1 de uno: la interfaz no repite esa aritmética.
    pub fn a1(fila: u32, col: u32) -> String {
        CellRef::new(fila, col).a1()
    }
    pub fn range_a1(r0: u32, c0: u32, r1: u32, c1: u32) -> String {
        RangeRef::new(CellRef::new(r0, c0), CellRef::new(r1, c1)).a1()
    }
}

impl Default for Hoja {
    fn default() -> Self {
        Hoja::new()
    }
}
