//! Docked panes: Layers and Browser on the left; Geoprocessing, Symbology, History and Details
//! on the right.

use std::collections::HashMap;

use egui::{Align2, CornerRadius, Rect, Sense, Stroke, Ui, pos2, vec2};
use rustgis_engine::Session;
use rustgis_engine::analysis::{ParamKind, tools};
use rustgis_engine::data::GeometryType;
use rustgis_engine::map::raster::RasterLegend;
use rustgis_engine::map::{ClassMethod, Layer, Marker, Ramp, RasterRenderer, Renderer, Resampling, Stretch, Symbol};
use serde_json::{Value, json};

use crate::theme::{Tokens, c32, regular, rgba, semibold};
use crate::{LeftPane, RightPane, RustGisApp, icons};

pub fn show(app: &mut RustGisApp, ui: &mut Ui) {
    let t = Tokens::get(ui.ctx());
    let frame = egui::Frame::NONE.fill(t.pane).inner_margin(egui::Margin::ZERO);
    if let Some(left) = app.ui.left {
        egui::Panel::left("left_pane").resizable(true).default_size(270.0).size_range(200.0..=480.0).frame(frame).show(ui, |ui| {
            let (title, close) = match left {
                LeftPane::Contents => ("Layers", "ui.pane.contents"),
                LeftPane::Catalog => ("Browser", "ui.pane.catalog"),
                LeftPane::Sources => ("Data Sources", "ui.pane.sources"),
            };
            pane_header(app, ui, tl!(title), close, &t);
            match left {
                LeftPane::Contents => contents(app, ui, &t),
                LeftPane::Catalog => catalog(app, ui, &t),
                LeftPane::Sources => crate::sourcespane::show(app, ui, &t),
            }
        });
    }
    if let Some(right) = app.ui.right {
        egui::Panel::right("right_pane").resizable(true).default_size(320.0).size_range(260.0..=560.0).frame(frame).show(ui, |ui| {
            let (title, close) = match right {
                RightPane::Geoprocessing => ("Tools", "ui.pane.geoprocessing"),
                RightPane::Symbology => ("Symbology", "ui.pane.symbology"),
                RightPane::History => ("History", "ui.pane.history"),
                RightPane::Details => ("Details", "ui.pane.details"),
                RightPane::Editing => ("Editing", "ui.pane.editing"),
                RightPane::Design => ("Data Design", "ui.pane.design"),
                RightPane::Workflows => ("Workflows", "ui.pane.workflows"),
                RightPane::Script => ("Script", "ui.pane.script"),
                RightPane::Tasks => ("Tasks", "ui.pane.tasks"),
                RightPane::Jobs => ("Workflow Manager", "ui.pane.jobs"),
                RightPane::Profile => ("Profile", "ui.pane.profile"),
                RightPane::Diagnostics => ("Diagnostic Monitor", "ui.pane.diagnostics"),
                RightPane::Infographics => ("Infographics", "ui.pane.infographics"),
                RightPane::Report => ("Report", "ui.pane.report"),
                RightPane::Presentation => ("Presentation", "ui.pane.presentation"),
                RightPane::Notebook => ("Notebook", "ui.pane.notebook"),
                RightPane::Assistant => ("AI Assistant", "ui.pane.assistant"),
                RightPane::Shares => ("Shared Items", "ui.pane.shares"),
                RightPane::Sql => ("SQL", "ui.pane.sql"),
            };
            pane_header(app, ui, tl!(title), close, &t);
            // The chat lays out its own scrolling, with the message box kept at the bottom.
            if right == RightPane::Assistant {
                crate::assistantpane::show(app, ui, &t);
                return;
            }
            // The SQL pane scrolls its own results sideways.
            if right == RightPane::Sql {
                crate::sqlpane::show(app, ui, &t);
                return;
            }
            egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
                egui::Frame::NONE.inner_margin(egui::Margin::symmetric(12, 8)).show(ui, |ui| match right {
                    RightPane::Geoprocessing => geoprocessing(app, ui, &t),
                    RightPane::Symbology => symbology(app, ui, &t),
                    RightPane::History => history(app, ui, &t),
                    RightPane::Details => details(app, ui, &t),
                    RightPane::Editing => crate::editpanes::editing(app, ui, &t),
                    RightPane::Design => crate::editpanes::design(app, ui, &t),
                    RightPane::Workflows => crate::analysispane::workflows(app, ui, &t),
                    RightPane::Script => crate::analysispane::script(app, ui, &t),
                    RightPane::Tasks => crate::workspacepanes::tasks(app, ui, &t),
                    RightPane::Jobs => crate::workspacepanes::jobs(app, ui, &t),
                    RightPane::Profile => crate::workspacepanes::profile(app, ui, &t),
                    RightPane::Diagnostics => crate::workspacepanes::diagnostics(app, ui, &t),
                    RightPane::Infographics => crate::workspacepanes::infographics(app, ui, &t),
                    RightPane::Report => crate::reportpane::report(app, ui, &t),
                    RightPane::Presentation => crate::docpanes::presentation(app, ui, &t),
                    RightPane::Notebook => crate::docpanes::notebook(app, ui, &t),
                    RightPane::Assistant | RightPane::Sql => {}
                    RightPane::Shares => crate::docpanes::shares(app, ui, &t),
                });
            });
        });
    }
}

fn pane_header(app: &mut RustGisApp, ui: &mut Ui, title: &str, close: &str, t: &Tokens) {
    let (r, _) = ui.allocate_exact_size(vec2(ui.available_width(), 34.0), Sense::hover());
    ui.painter().rect_filled(r, 0.0, t.pane_header);
    ui.painter().line_segment([pos2(r.min.x, r.max.y - 0.5), pos2(r.max.x, r.max.y - 0.5)], Stroke::new(1.0, t.border));
    ui.painter().text(pos2(r.min.x + 12.0, r.center().y), Align2::LEFT_CENTER, title, semibold(13.5), t.text);
    let cr = Rect::from_center_size(pos2(r.max.x - 18.0, r.center().y), vec2(22.0, 22.0));
    let resp = ui.interact(cr, egui::Id::new(("pane_close", title)), Sense::click());
    if resp.hovered() {
        ui.painter().rect_filled(cr, CornerRadius::same(4), t.hover);
    }
    icons::paint(ui.painter(), cr.shrink(5.0), "close", t.text_dim, t.accent);
    if resp.on_hover_text(tl!("Close")).clicked() {
        let _ = app.run(close, Value::Null);
    }
}

// ---------------------------------------------------------------------------------------------
// Contents

/// Paints a symbol patch for a layer of geometry type `g`.
pub fn patch(p: &egui::Painter, r: Rect, s: &Symbol, g: GeometryType) {
    match g {
        GeometryType::Point => {
            let rad = (s.size.clamp(4.0, 14.0)) / 2.0;
            let c = r.center();
            match s.marker {
                Marker::Square => {
                    p.rect_filled(Rect::from_center_size(c, vec2(rad * 1.8, rad * 1.8)), 0.0, c32(s.fill));
                    p.rect_stroke(
                        Rect::from_center_size(c, vec2(rad * 1.8, rad * 1.8)),
                        0.0,
                        Stroke::new(s.stroke_width.min(2.0), c32(s.stroke)),
                        egui::StrokeKind::Middle,
                    );
                }
                Marker::Triangle => {
                    p.add(egui::Shape::convex_polygon(
                        vec![c + vec2(0.0, -rad), c + vec2(rad, rad * 0.8), c + vec2(-rad, rad * 0.8)],
                        c32(s.fill),
                        Stroke::new(s.stroke_width.min(2.0), c32(s.stroke)),
                    ));
                }
                Marker::Diamond => {
                    p.add(egui::Shape::convex_polygon(
                        vec![c + vec2(0.0, -rad), c + vec2(rad, 0.0), c + vec2(0.0, rad), c + vec2(-rad, 0.0)],
                        c32(s.fill),
                        Stroke::new(s.stroke_width.min(2.0), c32(s.stroke)),
                    ));
                }
                _ => {
                    p.circle_filled(c, rad, c32(s.fill));
                    p.circle_stroke(c, rad, Stroke::new(s.stroke_width.min(2.0), c32(s.stroke)));
                }
            }
        }
        GeometryType::Polyline => {
            let a = pos2(r.min.x + 2.0, r.center().y + 3.0);
            let b = pos2(r.center().x, r.center().y - 3.0);
            let c = pos2(r.max.x - 2.0, r.center().y + 1.0);
            p.add(egui::Shape::line(vec![a, b, c], Stroke::new(s.stroke_width.clamp(1.0, 4.0), c32(s.stroke))));
        }
        _ => {
            let rr = r.shrink(2.0);
            p.rect_filled(rr, CornerRadius::same(2), c32(s.fill));
            p.rect_stroke(rr, CornerRadius::same(2), Stroke::new(s.stroke_width.clamp(0.5, 2.0), c32(s.stroke)), egui::StrokeKind::Inside);
        }
    }
}

fn contents(app: &mut RustGisApp, ui: &mut Ui, t: &Tokens) {
    // Drawing order header.
    ui.add_space(6.0);
    ui.horizontal(|ui| {
        ui.add_space(12.0);
        ui.label(egui::RichText::new(tl!("Drawing Order")).size(11.5).color(t.text_dim));
    });
    ui.add_space(2.0);
    egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
        let map_name = app.session.map().name.clone();
        ui.horizontal(|ui| {
            ui.add_space(8.0);
            let (r, _) = ui.allocate_exact_size(vec2(18.0, 18.0), Sense::hover());
            icons::paint(ui.painter(), r, "map", t.icon, t.accent);
            ui.label(egui::RichText::new(&map_name).font(semibold(12.5)));
        });
        let layers: Vec<Layer> = app.session.map().layers.clone();
        let scale = app.session.scale();
        let blocks: Vec<(u32, Rect)> = layers.iter().map(|l| (l.id, layer_row(app, ui, l, scale, t))).collect();
        layer_drop(app, ui, &blocks, t);
        // The basemap row at the bottom, like a desktop GIS.
        ui.add_space(2.0);
        ui.horizontal(|ui| {
            ui.add_space(30.0);
            let (r, _) = ui.allocate_exact_size(vec2(18.0, 18.0), Sense::hover());
            icons::paint(ui.painter(), r, "basemap", t.icon, t.accent);
            ui.label(
                egui::RichText::new(format!("{} · {}", tl!("Basemap"), tl!(app.session.map().basemap.label())))
                    .size(12.0)
                    .color(t.text_dim),
            );
        });
        if layers.is_empty() {
            ui.add_space(20.0);
            ui.vertical_centered(|ui| {
                ui.label(egui::RichText::new(tl!("This map has no layers yet.")).color(t.text_dim));
                if ui.button(tl!("Add Data…")).clicked() {
                    let _ = app.run("ui.addData", Value::Null);
                }
                if ui.button(tl!("Add sample countries")).clicked() {
                    let _ = app.run("layer.addSample", json!({"name": "countries"}));
                }
            });
        }
        // Standalone tables.
        let tables: Vec<(u32, String)> = app.session.project.tables.iter().map(|t| (t.id, t.name.clone())).collect();
        if !tables.is_empty() {
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                ui.add_space(12.0);
                ui.label(egui::RichText::new(tl!("Standalone Tables")).size(11.5).color(t.text_dim));
            });
            for (id, name) in tables {
                ui.horizontal(|ui| {
                    ui.add_space(30.0);
                    let (r, _) = ui.allocate_exact_size(vec2(16.0, 16.0), Sense::hover());
                    icons::paint(ui.painter(), r, "table", t.icon, t.accent);
                    let resp = ui.add(egui::Label::new(egui::RichText::new(&name).size(12.0)).sense(Sense::click()));
                    let mut open = resp.double_clicked() || resp.clicked();
                    resp.context_menu(|ui| {
                        if ui.button(tl!("Attribute Table")).clicked() {
                            open = true;
                            ui.close();
                        }
                        if ui.button(tl!("XY Table To Point…")).clicked() {
                            let _ = app.run("ui.gp.open", json!({"tool": "xyTableToPoint", "table": id}));
                            ui.close();
                        }
                        // Names can repeat: the id says which table. A view open on it closes itself.
                        if ui.button(tl!("Remove")).clicked() {
                            let _ = app.run("table.remove", json!({"table": id}));
                            ui.close();
                        }
                    });
                    if open {
                        app.session.table_standalone = Some(id);
                        app.session.table_layer = None;
                    }
                });
            }
        }
    });
}

/// The layer a row drag carries (egui's drag-and-drop payload).
#[derive(Clone, Copy)]
struct DraggedLayer(u32);

/// Dragging a layer row over the list: marks the gap it would drop into and, on release, moves
/// it there (`layer.move`, one undo step). Released anywhere but over the layer rows, the drag
/// does nothing.
fn layer_drop(app: &mut RustGisApp, ui: &Ui, blocks: &[(u32, Rect)], t: &Tokens) {
    let Some(dragged) = egui::DragAndDrop::payload::<DraggedLayer>(ui.ctx()) else { return };
    let Some(from) = blocks.iter().position(|(id, _)| *id == dragged.0) else { return };
    let view = ui.clip_rect();
    let Some(pointer) = ui.ctx().pointer_interact_pos().filter(|p| view.contains(*p)) else { return };
    // Near the top or bottom edge the list scrolls, faster the closer the pointer is.
    let edge = 28.0;
    let push = ((view.min.y + edge - pointer.y).max(0.0) - (pointer.y - (view.max.y - edge)).max(0.0)) / edge;
    if push != 0.0 {
        let dt = ui.input(|i| i.stable_dt).min(0.1);
        ui.scroll_with_delta_animation(vec2(0.0, push * 600.0 * dt), egui::style::ScrollAnimation::none());
        ui.ctx().request_repaint();
    }
    // Only the layer rows take a drop: not the map's name above them, the basemap or a table below.
    let (Some((_, first)), Some((_, last))) = (blocks.first(), blocks.last()) else { return };
    if pointer.y < first.min.y || pointer.y > last.max.y {
        return;
    }
    // The gap above the first row whose middle is below the pointer, or the one after the last.
    let gap = blocks.iter().position(|(_, r)| pointer.y < r.center().y).unwrap_or(blocks.len());
    // The gaps on either side of the dragged row leave it where it is.
    let to = if gap > from { gap - 1 } else { gap };
    if to == from {
        return;
    }
    let line = match blocks.get(gap) {
        Some((_, r)) => Some((r.min.y, r)),
        None => blocks.last().map(|(_, r)| (r.max.y, r)),
    };
    if let Some((y, r)) = line {
        ui.painter().line_segment([pos2(r.min.x + 8.0, y), pos2(r.max.x - 8.0, y)], Stroke::new(2.0, t.accent));
    }
    if ui.input(|i| i.pointer.any_released()) {
        let _ = app.run("layer.move", json!({"layer": dragged.0, "to": to}));
    }
}

/// A layer's row and, expanded, its legend; returns the rectangle of both.
fn layer_row(app: &mut RustGisApp, ui: &mut Ui, l: &Layer, scale: f64, t: &Tokens) -> Rect {
    let active = app.session.active_layer == Some(l.id);
    let row_h = 26.0;
    let (r, resp) = ui.allocate_exact_size(vec2(ui.available_width(), row_h), Sense::click_and_drag());
    if resp.drag_started_by(egui::PointerButton::Primary) {
        egui::DragAndDrop::set_payload(ui.ctx(), DraggedLayer(l.id));
    }
    let dragging = egui::DragAndDrop::payload::<DraggedLayer>(ui.ctx()).is_some_and(|d| d.0 == l.id);
    if active {
        ui.painter().rect_filled(r.shrink2(vec2(4.0, 1.0)), CornerRadius::same(5), t.selection);
    } else if resp.hovered() || dragging {
        ui.painter().rect_filled(r.shrink2(vec2(4.0, 1.0)), CornerRadius::same(5), t.hover);
    }
    let mut x = r.min.x + 10.0;
    // Expand arrow.
    let raster_legend = l.raster.as_ref().map(|r| r.legend());
    let has_legend = match &raster_legend {
        Some(rl) => !rl.is_empty(),
        None => !l.renderer.legend().is_empty(),
    };
    let ar = Rect::from_center_size(pos2(x + 6.0, r.center().y), vec2(14.0, 14.0));
    if has_legend {
        icons::paint(ui.painter(), ar, if l.expanded { "chevron_down" } else { "chevron_right" }, t.text_dim, t.accent);
    }
    let ar_resp = ui.interact(ar, egui::Id::new(("expand", l.id)), Sense::click());
    x += 16.0;
    // Visibility checkbox.
    let cb = Rect::from_center_size(pos2(x + 7.0, r.center().y), vec2(14.0, 14.0));
    let cb_resp = ui.interact(cb, egui::Id::new(("vis", l.id)), Sense::click());
    ui.painter().rect_filled(cb, CornerRadius::same(3), if l.visible { t.accent } else { t.input });
    ui.painter().rect_stroke(
        cb,
        CornerRadius::same(3),
        Stroke::new(1.0, if l.visible { t.accent } else { t.input_border }),
        egui::StrokeKind::Inside,
    );
    if l.visible {
        ui.painter().add(egui::Shape::line(
            vec![cb.min + vec2(3.0, 7.0), cb.min + vec2(6.0, 10.0), cb.min + vec2(11.0, 4.0)],
            Stroke::new(1.8, egui::Color32::WHITE),
        ));
    }
    x += 22.0;
    // Symbol patch (single symbol) or layer-type icon.
    let pr = Rect::from_center_size(pos2(x + 9.0, r.center().y), vec2(18.0, 16.0));
    if l.is_raster() {
        icons::paint(ui.painter(), pr, "raster_layer", t.icon, t.accent);
    } else if has_legend {
        let icon = match l.geometry_type() {
            GeometryType::Point => "point_layer",
            GeometryType::Polyline => "line_layer",
            _ => "polygon_layer",
        };
        icons::paint(ui.painter(), pr, icon, t.icon, t.accent);
    } else {
        patch(ui.painter(), pr, &l.renderer.base_symbol(), l.geometry_type());
    }
    x += 24.0;
    let out_of_range = !l.in_scale_range(scale);
    let missing = l.raster.as_ref().is_some_and(|r| r.data.is_none()) || l.source.as_ref().is_some_and(|s| s.error.is_some());
    let color = if missing {
        t.red
    } else if !l.visible || out_of_range {
        t.text_disabled
    } else {
        t.text
    };
    let name = if l.definition_query.is_some() { format!("{}  {}", l.name, tl!("(filtered)")) } else { l.name.clone() };
    ui.painter().text(pos2(x, r.center().y), Align2::LEFT_CENTER, &name, if active { semibold(12.5) } else { regular(12.5) }, color);
    if !l.selection.is_empty() {
        let badge = format!("{}", l.selection.len());
        let bw = crate::widgets::text_w(ui, &badge, regular(10.5)) + 10.0;
        let br = Rect::from_center_size(pos2(r.max.x - 12.0 - bw / 2.0, r.center().y), vec2(bw, 16.0));
        ui.painter().rect_filled(br, CornerRadius::same(8), egui::Color32::from_rgb(0x00, 0xA8, 0xC0));
        ui.painter().text(br.center(), Align2::CENTER_CENTER, badge, regular(10.5), egui::Color32::WHITE);
    }
    if cb_resp.clicked() {
        let _ = app.run("layer.visibility", json!({"layer": l.id}));
    } else if ar_resp.clicked() {
        let _ = app.run("layer.expand", json!({"layer": l.id}));
    } else if resp.clicked() {
        let _ = app.run("layer.select", json!({"layer": l.id}));
    }
    if resp.double_clicked() {
        let _ = app.run("layer.select", json!({"layer": l.id}));
        app.ui.right = Some(RightPane::Symbology);
    }
    let tip = match &l.raster {
        Some(rl) => match &rl.data {
            Some(d) => crate::i18n::fmt(
                "Raster · {w} × {h} · {n} bands · {renderer}",
                &[
                    ("w", &d.width.to_string()),
                    ("h", &d.height.to_string()),
                    ("n", &d.band_count().to_string()),
                    ("renderer", tl!(rl.style.renderer.label())),
                ],
            ),
            None => crate::i18n::fmt("The raster's data couldn't be read: {path}", &[("path", rl.path.as_deref().unwrap_or(""))]),
        },
        None => match &l.source {
            Some(src) if src.error.is_some() => {
                crate::i18n::fmt("The layer's file couldn't be read: {error}", &[("error", src.error.as_deref().unwrap_or(""))])
            }
            Some(src) => {
                let format = src.info().map(|i| i.format).unwrap_or("");
                let total = src.info().and_then(|i| i.count).map(|c| c.to_string()).unwrap_or_else(|| "?".into());
                match src.sampled() {
                    Some((n, matched)) => crate::i18n::fmt(
                        "{type} · {format}, read in place · a sample of {n} of the {matched} features in view (zoom in to see them all) · {renderer}",
                        &[
                            ("type", tl!(l.geometry_type().label())),
                            ("format", format),
                            ("n", &n.to_string()),
                            ("matched", &matched.to_string()),
                            ("renderer", tl!(l.renderer.kind_label())),
                        ],
                    ),
                    None => crate::i18n::fmt(
                        "{type} · {format}, read in place · {n} of {total} features in view · {renderer}",
                        &[
                            ("type", tl!(l.geometry_type().label())),
                            ("format", format),
                            ("n", &l.dataset.features.len().to_string()),
                            ("total", &total),
                            ("renderer", tl!(l.renderer.kind_label())),
                        ],
                    ),
                }
            }
            None => crate::i18n::fmt(
                "{type} · {n} features · {renderer}",
                &[
                    ("type", tl!(l.geometry_type().label())),
                    ("n", &l.dataset.features.len().to_string()),
                    ("renderer", tl!(l.renderer.kind_label())),
                ],
            ),
        },
    };
    let resp = resp.on_hover_text(tip);
    resp.context_menu(|ui| {
        let _ = app.session.run("layer.select", &json!({"layer": l.id}));
        let items: &[(&str, &str)] = if l.is_raster() {
            &[("Zoom To Layer", "layer.zoomTo"), ("Symbology", "ui.pane.symbology")]
        } else {
            &[("Zoom To Layer", "layer.zoomTo"), ("Attribute Table", "table.open"), ("Symbology", "ui.pane.symbology")]
        };
        for &(label, id) in items {
            if ui.button(tl!(label)).clicked() {
                let p = if id.starts_with("ui.") { Value::Null } else { json!({"layer": l.id}) };
                if id == "ui.pane.symbology" {
                    app.ui.right = Some(RightPane::Symbology);
                } else {
                    let _ = app.run(id, p);
                }
                ui.close();
            }
        }
        if !l.is_raster() && ui.button(tl!(if l.labels.enabled { "Turn Labels Off" } else { "Label" })).clicked() {
            let _ = app.run("labels.enable", json!({"layer": l.id}));
            ui.close();
        }
        if l.is_streamed() && ui.button(tl!("Load All Features")).clicked() {
            let _ = app.run("layer.loadAll", json!({"layer": l.id}));
            ui.close();
        }
        ui.separator();
        if ui.button(tl!("Move Up")).clicked() {
            let _ = app.run("layer.moveUp", json!({"layer": l.id}));
            ui.close();
        }
        if ui.button(tl!("Move Down")).clicked() {
            let _ = app.run("layer.moveDown", json!({"layer": l.id}));
            ui.close();
        }
        if ui.button(tl!("Rename…")).clicked() {
            let _ = app.run("ui.dialog", json!({"name": "renameLayer"}));
            ui.close();
        }
        if l.is_raster() {
            if ui.button(tl!("Export Raster…")).clicked() {
                let _ = app.run("ui.exportRaster", Value::Null);
                ui.close();
            }
        } else {
            if ui.button(tl!("Definition Query…")).clicked() {
                let _ = app.run("ui.dialog", json!({"name": "definitionQuery"}));
                ui.close();
            }
            if ui.button(tl!("Export Features…")).clicked() {
                let _ = app.run("ui.exportLayer", Value::Null);
                ui.close();
            }
        }
        ui.separator();
        if ui.button(tl!("Remove")).clicked() {
            let _ = app.run("layer.remove", json!({"layer": l.id}));
            ui.close();
        }
    });
    // Legend classes.
    if l.expanded
        && let Some(entries) = &raster_legend
    {
        raster_legend_rows(ui, entries, 74.0, t);
    } else if l.expanded && has_legend {
        let field = l.renderer.field().unwrap_or("").to_string();
        ui.horizontal(|ui| {
            ui.add_space(74.0);
            ui.label(egui::RichText::new(field).size(11.0).color(t.text_dim));
        });
        for e in l.renderer.legend().iter().take(40) {
            let (r, _) = ui.allocate_exact_size(vec2(ui.available_width(), 20.0), Sense::hover());
            let pr = Rect::from_center_size(pos2(r.min.x + 84.0, r.center().y), vec2(18.0, 14.0));
            patch(ui.painter(), pr, &e.symbol, l.geometry_type());
            ui.painter().text(pos2(r.min.x + 100.0, r.center().y), Align2::LEFT_CENTER, &e.label, regular(11.5), t.text);
        }
    }
    Rect::from_min_max(r.min, pos2(r.max.x, ui.min_rect().max.y.max(r.max.y)))
}

// ---------------------------------------------------------------------------------------------
// Catalog

fn catalog(app: &mut RustGisApp, ui: &mut Ui, t: &Tokens) {
    egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
        egui::Frame::NONE.inner_margin(egui::Margin::symmetric(10, 8)).show(ui, |ui| {
            egui::CollapsingHeader::new(egui::RichText::new(tl!("Maps")).font(semibold(12.5))).default_open(true).show(ui, |ui| {
                let maps: Vec<String> = app.session.project.maps.iter().map(|m| m.name.clone()).collect();
                for (i, m) in maps.iter().enumerate() {
                    ui.horizontal(|ui| {
                        let (r, _) = ui.allocate_exact_size(vec2(16.0, 16.0), Sense::hover());
                        icons::paint(ui.painter(), r, "map", t.icon, t.accent);
                        let active = i == app.session.project.active_map;
                        let resp = ui.selectable_label(active, m);
                        if resp.clicked() {
                            let _ = app.run("map.activate", json!({"name": m}));
                        }
                        resp.context_menu(|ui| {
                            if ui.button(tl!("Delete")).clicked() {
                                let _ = app.run("map.remove", json!({"name": m}));
                                ui.close();
                            }
                        });
                    });
                }
            });
            egui::CollapsingHeader::new(egui::RichText::new(tl!("Toolboxes")).font(semibold(12.5))).default_open(true).show(ui, |ui| {
                let mut boxes: Vec<&str> = tools().map(|t| t.toolbox.split(" › ").next().unwrap_or(t.toolbox)).collect();
                boxes.sort();
                boxes.dedup();
                let link = |app: &mut RustGisApp, ui: &mut Ui, tool: &rustgis_engine::analysis::ToolSpec| {
                    ui.horizontal(|ui| {
                        let (r, _) = ui.allocate_exact_size(vec2(16.0, 16.0), Sense::hover());
                        icons::paint(ui.painter(), r, "tools", t.icon, t.accent);
                        if ui.link(tl!(tool.name)).on_hover_text(tl!(tool.summary)).clicked() {
                            let _ = app.run("ui.gp.open", json!({"tool": tool.id}));
                        }
                    });
                };
                for b in boxes {
                    egui::CollapsingHeader::new(crate::i18n::location(b)).default_open(false).show(ui, |ui| {
                        let in_box: Vec<&rustgis_engine::analysis::ToolSpec> =
                            tools().filter(|x| x.toolbox.split(" › ").next() == Some(b)).collect();
                        // A large toolbox (the GeoLibre suite) opens one level at a time:
                        // `GeoLibre › Raster › General`.
                        if in_box.len() > 40 {
                            toolbox_levels(app, ui, &in_box, 1, &link);
                        } else {
                            for tool in in_box {
                                link(app, ui, tool);
                            }
                        }
                    });
                }
            });
            let layouts: Vec<String> = app.session.project.layouts.iter().map(|l| l.name.clone()).collect();
            if !layouts.is_empty() {
                egui::CollapsingHeader::new(egui::RichText::new(tl!("Layouts")).font(semibold(12.5))).default_open(true).show(ui, |ui| {
                    for n in layouts {
                        ui.horizontal(|ui| {
                            let (r, _) = ui.allocate_exact_size(vec2(16.0, 16.0), Sense::hover());
                            icons::paint(ui.painter(), r, "new_layout", t.icon, t.accent);
                            let resp = ui.selectable_label(
                                app.session
                                    .project
                                    .layouts
                                    .get(app.session.active_layout.unwrap_or(usize::MAX))
                                    .is_some_and(|l| l.name == n),
                                &n,
                            );
                            if resp.clicked() {
                                let _ = app.run("layout.activate", json!({"name": n}));
                                app.ui.tab = "Publish".into();
                            }
                            resp.context_menu(|ui| {
                                if ui.button(tl!("Delete")).clicked() {
                                    let _ = app.run("layout.remove", json!({"name": n}));
                                    ui.close();
                                }
                            });
                        });
                    }
                });
            }
            let styles: Vec<String> = app.session.project.styles.iter().map(|st| st.name.clone()).collect();
            if !styles.is_empty() {
                egui::CollapsingHeader::new(egui::RichText::new(tl!("Styles")).font(semibold(12.5))).default_open(true).show(ui, |ui| {
                    for n in styles {
                        ui.horizontal(|ui| {
                            let (r, _) = ui.allocate_exact_size(vec2(16.0, 16.0), Sense::hover());
                            icons::paint(ui.painter(), r, "symbology", t.icon, t.accent);
                            let resp = ui.link(&n).on_hover_text(tl!("Apply to the selected layer"));
                            if resp.clicked() {
                                let _ = app.run("style.apply", json!({"name": n}));
                            }
                            resp.context_menu(|ui| {
                                if ui.button(tl!("Delete")).clicked() {
                                    let _ = app.run("style.remove", json!({"name": n}));
                                    ui.close();
                                }
                            });
                        });
                    }
                });
            }
            let ramps: Vec<Ramp> = app.session.project.ramps.iter().map(|r| Ramp::Custom(Box::new(r.clone()))).collect();
            if !ramps.is_empty() {
                egui::CollapsingHeader::new(egui::RichText::new(tl!("Color Ramps")).font(semibold(12.5))).default_open(true).show(
                    ui,
                    |ui| {
                        for r in ramps {
                            ui.horizontal(|ui| {
                                ramp_swatch(ui, &r, 16.0, false, t);
                                let resp = ui.link(r.name()).on_hover_text(tl!("Apply to the selected layer"));
                                if resp.clicked() {
                                    let _ = app.run("layer.ramp", json!({"ramp": r.name()}));
                                }
                                resp.context_menu(|ui| {
                                    if ui.button(tl!("Delete")).clicked() {
                                        let _ = app.run("ramp.remove", json!({"name": r.name()}));
                                        ui.close();
                                    }
                                });
                            });
                        }
                    },
                );
            }
            let tasks: Vec<(String, usize)> = app.session.project.tasks.iter().map(|tk| (tk.name.clone(), tk.steps.len())).collect();
            if !tasks.is_empty() {
                egui::CollapsingHeader::new(egui::RichText::new(tl!("Tasks")).font(semibold(12.5))).default_open(true).show(ui, |ui| {
                    for (n, steps) in tasks {
                        ui.horizontal(|ui| {
                            let (r, _) = ui.allocate_exact_size(vec2(16.0, 16.0), Sense::hover());
                            icons::paint(ui.painter(), r, "run", t.icon, t.accent);
                            let resp = ui
                                .link(crate::i18n::fmt("{name} ({n} steps)", &[("name", &n), ("n", &steps.to_string())]))
                                .on_hover_text(tl!("Run the task"));
                            if resp.clicked() {
                                let _ = app.run("task.run", json!({"task": n}));
                            }
                            resp.context_menu(|ui| {
                                if ui.button(tl!("Delete")).clicked() {
                                    let _ = app.run("task.remove", json!({"task": n}));
                                    ui.close();
                                }
                            });
                        });
                    }
                });
            }
            let folders = app.session.project.folders.clone();
            if !folders.is_empty() && !cfg!(target_arch = "wasm32") {
                egui::CollapsingHeader::new(egui::RichText::new(tl!("Folder Connections")).font(semibold(12.5))).default_open(true).show(
                    ui,
                    |ui| {
                        for f in folders {
                            ui.horizontal(|ui| {
                                let (r, _) = ui.allocate_exact_size(vec2(16.0, 16.0), Sense::hover());
                                icons::paint(ui.painter(), r, "add_folder", t.icon, t.accent);
                                let resp = ui.link(&f).on_hover_text(tl!("Browse this folder below"));
                                if resp.clicked() {
                                    app.table.folder = Some(std::path::PathBuf::from(&f));
                                }
                                resp.context_menu(|ui| {
                                    if ui.button(tl!("Remove")).clicked() {
                                        let _ = app.run("project.removeFolder", json!({"path": f}));
                                        ui.close();
                                    }
                                });
                            });
                        }
                    },
                );
            }
            egui::CollapsingHeader::new(egui::RichText::new(tl!("Tables")).font(semibold(12.5))).default_open(true).show(ui, |ui| {
                let tables: Vec<(u32, String)> = app.session.project.tables.iter().map(|t| (t.id, t.name.clone())).collect();
                if tables.is_empty() {
                    ui.label(egui::RichText::new(tl!("No standalone tables")).small().weak());
                }
                for (id, n) in tables {
                    if ui.link(&n).clicked() {
                        app.session.table_standalone = Some(id);
                        app.session.table_layer = None;
                    }
                }
            });
            // A web page can't list local folders.
            if !cfg!(target_arch = "wasm32") {
                egui::CollapsingHeader::new(egui::RichText::new(tl!("Folders")).font(semibold(12.5))).default_open(true).show(ui, |ui| {
                    folder_browser(app, ui, t);
                });
            }
        });
    });
}

fn folder_browser(app: &mut RustGisApp, ui: &mut Ui, t: &Tokens) {
    let dir = app
        .table
        .folder
        .clone()
        .or_else(|| app.session.path.as_ref().and_then(|p| p.parent().map(|d| d.to_path_buf())))
        .or_else(|| std::env::current_dir().ok());
    let Some(dir) = dir else { return };
    ui.horizontal(|ui| {
        if ui.small_button("↑").on_hover_text(tl!("Parent folder")).clicked()
            && let Some(p) = dir.parent()
        {
            app.table.folder = Some(p.to_path_buf());
        }
        ui.label(egui::RichText::new(dir.display().to_string()).small().weak());
    });
    let Ok(rd) = std::fs::read_dir(&dir) else {
        ui.label(egui::RichText::new(tl!("Can't read this folder")).small().weak());
        return;
    };
    let mut entries: Vec<(bool, String, std::path::PathBuf)> = rd
        .filter_map(|e| e.ok())
        .map(|e| (e.path().is_dir(), e.file_name().to_string_lossy().to_string(), e.path()))
        .filter(|(d, n, _)| {
            !n.starts_with('.')
                && (*d
                    || [
                        "geojson",
                        "json",
                        "csv",
                        "tsv",
                        "shp",
                        "tif",
                        "tiff",
                        "asc",
                        "rgis",
                        "rgisx",
                        "rgispkg",
                        "rgismap",
                        "rgismpk",
                        "rgislyr",
                        "rgislayout",
                    ]
                    .iter()
                    .any(|x| n.to_lowercase().ends_with(&format!(".{x}"))))
        })
        .collect();
    entries.sort_by(|a, b| b.0.cmp(&a.0).then(a.1.to_lowercase().cmp(&b.1.to_lowercase())));
    for (is_dir, name, path) in entries.into_iter().take(300) {
        ui.horizontal(|ui| {
            let (r, _) = ui.allocate_exact_size(vec2(16.0, 16.0), Sense::hover());
            let icon = if is_dir {
                "add_folder"
            } else if rustgis_engine::project::is_project_file(&name) {
                "logo"
            } else {
                "add_data"
            };
            icons::paint(ui.painter(), r, icon, t.icon, t.accent);
            let resp = ui.add(egui::Label::new(egui::RichText::new(&name).size(12.0)).sense(Sense::click()));
            let resp = resp.on_hover_text(tl!(if is_dir { "Open folder" } else { "Double-click to add" }));
            if resp.double_clicked() || (is_dir && resp.clicked()) {
                if is_dir {
                    app.table.folder = Some(path.clone());
                } else if rustgis_engine::project::is_project_file(&name) || name.ends_with(".rgispkg") {
                    let _ = app.run("project.open", json!({"path": path.display().to_string()}));
                    app.map.invalidate();
                } else if name.ends_with(".rgismap") || name.ends_with(".rgismpk") {
                    let _ = app.run("map.import", json!({"path": path.display().to_string()}));
                    app.map.invalidate();
                } else if name.ends_with(".rgislayout") {
                    let _ = app.run("layout.import", json!({"path": path.display().to_string()}));
                } else {
                    let _ = app.run("layer.addData", json!({"path": path.display().to_string()}));
                }
            }
        });
    }
}

/// `tools` grouped by the `depth`-th part of their toolbox (`GeoLibre › Raster › General`), one
/// closed header per group and nested again below it; tools whose toolbox stops here are listed.
fn toolbox_levels(
    app: &mut RustGisApp,
    ui: &mut Ui,
    tools: &[&'static rustgis_engine::analysis::ToolSpec],
    depth: usize,
    link: &dyn Fn(&mut RustGisApp, &mut Ui, &rustgis_engine::analysis::ToolSpec),
) {
    let part = |x: &rustgis_engine::analysis::ToolSpec| x.toolbox.split(" › ").nth(depth);
    let mut groups: Vec<&str> = tools.iter().filter_map(|x| part(x)).collect();
    groups.sort();
    groups.dedup();
    for g in groups {
        let inside: Vec<_> = tools.iter().copied().filter(|x| part(x) == Some(g)).collect();
        let path: Vec<&str> = inside.first().map(|x| x.toolbox.split(" › ").take(depth + 1).collect()).unwrap_or_default();
        egui::CollapsingHeader::new(crate::i18n::location(g)).id_salt(("toolbox", path)).default_open(false).show(ui, |ui| {
            toolbox_levels(app, ui, &inside, depth + 1, link);
        });
    }
    for tool in tools.iter().filter(|x| part(x).is_none()) {
        link(app, ui, tool);
    }
}

// ---------------------------------------------------------------------------------------------
// Geoprocessing

#[derive(Default)]
pub struct GpForm {
    pub tool: Option<&'static str>,
    pub values: HashMap<&'static str, String>,
    /// The standalone table picked for an input, by its id (names can repeat); the input's value
    /// is then the table's name, for show.
    pub tables: HashMap<&'static str, u32>,
    pub search: String,
    pub last: Option<(bool, Vec<String>)>,
    /// The fields that can hold coordinates, for the project revision and the input (its name,
    /// and its id for a table) they were read from: telling them reads rows, too much to do
    /// every frame.
    pub coordinate_fields: Option<((u64, String, Option<u32>), Vec<String>)>,
    /// A GeoLibre runner install in progress (it runs on a background thread).
    pub install: Option<std::sync::Arc<std::sync::Mutex<Install>>>,
    /// The browser's GeoLibre tools loading.
    pub web_load: Option<crate::Slot<Result<String, String>>>,
    /// A tool running in the browser's GeoLibre runner: its run id, and where the result lands.
    pub web_run: Option<(u64, &'static str, crate::Slot<rustgis_engine::cmd::gp::ExternalOutput>)>,
    /// Uninstall was pressed once: the button now asks to confirm.
    pub confirm_uninstall: bool,
    /// Why the last uninstall failed (apart from `last`, which a tool run sets too).
    pub uninstall_error: Option<String>,
}

/// Starts loading the browser's GeoLibre tools (the runner downloads once, then the browser
/// caches it).
pub fn start_geolibre_load(app: &mut RustGisApp) {
    if app.gp.web_load.is_some() {
        return;
    }
    let Some(load) = &app.services.geolibre_manifests else { return };
    let slot = crate::Slot::default();
    load(slot.clone());
    app.gp.web_load = Some(slot);
}

fn take<T>(slot: &crate::Slot<T>) -> Option<T> {
    slot.lock().ok().and_then(|mut s| s.take())
}

/// Picks up what the browser's GeoLibre runner finished: the tool list, or a tool run.
pub fn poll_geolibre(app: &mut RustGisApp, ctx: &egui::Context) {
    if let Some(r) = app.gp.web_load.as_ref().and_then(take) {
        app.gp.web_load = None;
        match r.and_then(|json| rustgis_engine::cmd::gp::external_load_manifests(&json)) {
            Ok(_) => {
                if let Some(save) = &app.services.save_pref {
                    save("geolibre_web", "on");
                }
            }
            Err(e) => app.gp.last = Some((false, vec![e])),
        }
    }
    if let Some((id, tool, out)) = app.gp.web_run.as_ref().and_then(|(id, tool, slot)| take(slot).map(|o| (*id, *tool, o))) {
        app.gp.web_run = None;
        let r = rustgis_engine::cmd::gp::external_complete(&mut app.session, id, out).and_then(|_| app.run("gp.finish", json!({"id": id})));
        let last = match r {
            Ok(_) => (true, app.session.project.gp_history.last().map(|h| h.messages.clone()).unwrap_or_default()),
            Err(e) => (false, vec![e]),
        };
        // Under its own form only; with another tool open, a notification says how it went.
        if app.gp.tool == Some(tool) {
            app.gp.last = Some(last);
        } else if !last.0 {
            app.session.notify(last.1.join("\n"), true);
        } else {
            let name = rustgis_engine::analysis::tool(tool).map(|t| t.name).unwrap_or(tool);
            app.session.notify(crate::i18n::fmt("Finished: {tool}", &[("tool", tl!(name))]), false);
        }
        app.map.invalidate();
    }
    if app.gp.web_load.is_some() || app.gp.web_run.is_some() {
        ctx.request_repaint_after(std::time::Duration::from_millis(100));
    }
}

/// Progress of a one-click GeoLibre runner install.
#[derive(Default)]
pub struct Install {
    pub done: u64,
    pub total: Option<u64>,
    /// The installed program, or why it failed; set when the install ends.
    pub result: Option<Result<String, String>>,
}

impl GpForm {
    /// Opens a tool, pre-filling layer parameters from the active layer.
    pub fn open(&mut self, tool: Option<&'static str>, s: &Session) {
        self.tool = tool;
        self.values.clear();
        self.tables.clear();
        self.last = None;
        let Some(spec) = tool.and_then(rustgis_engine::analysis::tool) else { return };
        let active = s.active_layer.and_then(|id| s.map().layer(id));
        for p in spec.params {
            let v = match p.kind {
                ParamKind::Layer => active.map(|l| l.name.clone()),
                ParamKind::PolygonLayer => active
                    .filter(|l| l.geometry_type() == GeometryType::Polygon)
                    .or_else(|| s.map().layers.iter().find(|l| l.geometry_type() == GeometryType::Polygon))
                    .map(|l| l.name.clone()),
                ParamKind::PointLayer => active
                    .filter(|l| l.geometry_type() == GeometryType::Point)
                    .or_else(|| s.map().layers.iter().find(|l| l.geometry_type() == GeometryType::Point))
                    .map(|l| l.name.clone()),
                ParamKind::RasterLayer => {
                    active.filter(|l| l.is_raster()).or_else(|| s.map().layers.iter().find(|l| l.is_raster())).map(|l| l.name.clone())
                }
                ParamKind::VectorLayer => {
                    active.filter(|l| !l.is_raster()).or_else(|| s.map().layers.iter().find(|l| !l.is_raster())).map(|l| l.name.clone())
                }
                _ => serde_json::from_str::<Value>(p.default).ok().map(|v| v.as_str().map(str::to_string).unwrap_or_else(|| v.to_string())),
            };
            if let Some(v) = v {
                self.values.insert(p.name, v);
            }
        }
    }

    /// Puts a standalone table, by its id, in the open tool's first input that takes one.
    pub fn pick_table(&mut self, id: u32, s: &Session) {
        let spec = self.tool.and_then(rustgis_engine::analysis::tool);
        let input = spec.and_then(|t| t.params.iter().find(|p| p.kind == ParamKind::Layer));
        if let (Some(p), Some(t)) = (input, s.project.table(id)) {
            self.values.insert(p.name, t.name.clone());
            self.tables.insert(p.name, id);
        }
    }
}

/// Where to get the GeoLibre tools when the runner isn't installed (desktop and CLI builds only).
fn geolibre_card(app: &mut RustGisApp, ui: &mut Ui, t: &Tokens) {
    // Read directly, not through Session::run: a command each frame would bump the session's
    // revision and make the views redraw. The status itself is cached in the runner module.
    let status = rustgis_engine::cmd::gp::geolibre_status();
    if status["supported"] != true {
        return;
    }
    // An install that just finished: use the runner, and remember it.
    let finished = app.gp.install.as_ref().and_then(|st| st.lock().ok().and_then(|mut s| s.result.take()));
    if let Some(result) = finished {
        app.gp.install = None;
        match result.and_then(|path| app.run("gp.setGeolibre", json!({"path": path})).map(|_| path)) {
            Ok(path) => {
                if let Some(save) = &app.services.save_pref {
                    save("geolibre_runner", &path);
                }
                app.gp.last = None;
            }
            Err(e) => app.gp.last = Some((false, vec![e])),
        }
        return;
    }
    if status["web"] == true {
        geolibre_web_card(app, ui, t);
        return;
    }
    // An installed runner that lists no tools (broken, or a failed download of its list): it
    // can still be removed from here.
    if status["installed"] == true {
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new(tl!("The installed GeoLibre runner lists no tools.")).size(11.0).color(t.text_dim));
            geolibre_uninstall(app, ui, t);
        });
        if let Some(e) = &app.gp.uninstall_error {
            ui.label(egui::RichText::new(e).size(11.0).color(t.red));
        }
    }
    let download = status["download"].as_str().unwrap_or("https://github.com/opengeos/geolibre-rust/releases/latest").to_string();
    ui.add_space(6.0);
    egui::Frame::NONE.stroke(Stroke::new(1.0, t.border)).corner_radius(CornerRadius::same(6)).inner_margin(egui::Margin::same(10)).show(
        ui,
        |ui| {
            ui.set_width(ui.available_width());
            ui.label(egui::RichText::new(tl!("GeoLibre tools")).font(semibold(13.0)));
            ui.label(
                egui::RichText::new(tl!(
                    "Over 1,000 raster, vector, LiDAR, hydrology and terrain tools, from an optional download of about 20 MB."
                ))
                .size(11.5)
                .color(t.text_dim),
            );
            ui.add_space(6.0);
            let progress = app.gp.install.as_ref().and_then(|st| st.lock().ok().map(|s| (s.done, s.total)));
            match progress {
                Some((done, total)) => {
                    let mb = |b: u64| format!("{:.1}", b as f64 / 1_048_576.0);
                    let text = crate::i18n::fmt("Downloading… {mb} MB", &[("mb", &mb(done))]);
                    let bar = egui::ProgressBar::new(total.map(|t| done as f32 / t.max(1) as f32).unwrap_or(0.0)).text(text);
                    ui.add(if total.is_some() { bar } else { bar.animate(true) });
                    // Keep drawing while the download runs on its thread.
                    ui.ctx().request_repaint_after(std::time::Duration::from_millis(100));
                }
                None => {
                    #[cfg(not(target_arch = "wasm32"))]
                    if crate::widgets::primary_button(ui, tl!("Install GeoLibre tools"), true, vec2(0.0, 28.0)).clicked() {
                        let st = std::sync::Arc::new(std::sync::Mutex::new(Install::default()));
                        app.gp.install = Some(st.clone());
                        app.gp.last = None;
                        let ctx = ui.ctx().clone();
                        std::thread::spawn(move || {
                            let result = rustgis_engine::cmd::gp::install_geolibre(&|done, total| {
                                if let Ok(mut s) = st.lock() {
                                    s.done = done;
                                    s.total = total;
                                }
                            });
                            if let Ok(mut s) = st.lock() {
                                s.result = Some(result);
                            }
                            ctx.request_repaint();
                        });
                    }
                    ui.add_space(2.0);
                    ui.horizontal_wrapped(|ui| {
                        ui.label(egui::RichText::new(tl!("Or")).size(11.0).color(t.text_dim));
                        ui.add(egui::Hyperlink::from_label_and_url(tl!("Download the GeoLibre runner"), download).open_in_new_tab(true));
                        #[cfg(not(target_arch = "wasm32"))]
                        if ui.small_button(tl!("Locate…")).clicked()
                            && let Some(path) = rfd::FileDialog::new().set_title(tl!("Locate the GeoLibre runner")).pick_file()
                        {
                            let path = path.display().to_string();
                            match app.run("gp.setGeolibre", json!({"path": path})) {
                                Ok(_) => {
                                    if let Some(save) = &app.services.save_pref {
                                        save("geolibre_runner", &path);
                                    }
                                    app.gp.last = None;
                                }
                                Err(e) => app.gp.last = Some((false, vec![e])),
                            }
                        }
                    });
                }
            }
            if let Some((false, msgs)) = &app.gp.last {
                for m in msgs {
                    ui.label(egui::RichText::new(m).size(11.0).color(t.red));
                }
            }
        },
    );
}

/// Uninstalls the GeoLibre tools (a second press confirms): natively the runner that Install
/// put in place (not one of the user's own, found with Locate… or on the `PATH`); in the
/// browser, the loaded tools, which then stop loading at startup.
fn geolibre_uninstall(app: &mut RustGisApp, ui: &mut Ui, t: &Tokens) {
    let status = rustgis_engine::cmd::gp::geolibre_status();
    let web = status["web"] == true;
    if !web && status["installed"] != true {
        return;
    }
    let label = match (web, app.gp.confirm_uninstall) {
        (_, true) => tl!("Confirm uninstall"),
        (true, false) => tl!("Remove"),
        (false, false) => tl!("Uninstall"),
    };
    let tip = if web {
        tl!("Unload the GeoLibre tools and stop loading them when RustGIS starts.")
    } else {
        tl!("Delete the GeoLibre runner that RustGIS installed. Install it again at any time.")
    };
    let text = egui::RichText::new(label).size(11.0);
    let resp = ui.small_button(if app.gp.confirm_uninstall { text.color(t.red) } else { text }).on_hover_text(tip);
    if !resp.clicked() {
        // A click elsewhere takes back the first press.
        if app.gp.confirm_uninstall && ui.input(|i| i.pointer.any_click()) && !resp.hovered() {
            app.gp.confirm_uninstall = false;
        }
        return;
    }
    if !app.gp.confirm_uninstall {
        app.gp.confirm_uninstall = true;
        return;
    }
    app.gp.confirm_uninstall = false;
    match app.run("gp.uninstallGeolibre", Value::Null) {
        Ok(_) => {
            if let Some(save) = &app.services.save_pref {
                // Forget the runner Install remembered (or, in the browser, the loaded tools).
                if web {
                    save("geolibre_web", "off");
                } else {
                    save("geolibre_runner", "");
                }
            }
            app.gp.uninstall_error = None;
        }
        Err(e) => app.gp.uninstall_error = Some(e),
    }
}

/// The browser's version of the card: load the WASI runner's tools.
fn geolibre_web_card(app: &mut RustGisApp, ui: &mut Ui, t: &Tokens) {
    ui.add_space(6.0);
    egui::Frame::NONE
        .stroke(Stroke::new(1.0, t.border))
        .corner_radius(CornerRadius::same(6))
        .inner_margin(egui::Margin::same(10))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.label(egui::RichText::new(tl!("GeoLibre tools")).font(semibold(13.0)));
            ui.label(
                egui::RichText::new(tl!(
                    "Over 1,000 raster, vector, LiDAR, hydrology and terrain tools that run in the browser, from a one-time download of about 25 MB."
                ))
                .size(11.5)
                .color(t.text_dim),
            );
            ui.add_space(6.0);
            if app.gp.web_load.is_some() {
                ui.horizontal(|ui| {
                    ui.spinner();
                    ui.label(egui::RichText::new(tl!("Loading the GeoLibre tools…")).size(11.5));
                });
            } else if crate::widgets::primary_button(ui, tl!("Load GeoLibre tools"), true, vec2(0.0, 28.0)).clicked() {
                app.gp.last = None;
                start_geolibre_load(app);
            }
            if let Some((false, msgs)) = &app.gp.last {
                for m in msgs {
                    ui.label(egui::RichText::new(m).size(11.0).color(t.red));
                }
            }
        });
}

fn geoprocessing(app: &mut RustGisApp, ui: &mut Ui, t: &Tokens) {
    let Some(spec) = app.gp.tool.and_then(rustgis_engine::analysis::tool) else {
        // Tool finder.
        ui.horizontal(|ui| {
            let (r, _) = ui.allocate_exact_size(vec2(16.0, 16.0), Sense::hover());
            icons::paint(ui.painter(), r, "search", t.text_dim, t.accent);
            ui.add(egui::TextEdit::singleline(&mut app.gp.search).hint_text(tl!("Find Tools")).desired_width(f32::INFINITY));
        });
        ui.add_space(8.0);
        let q = app.gp.search.to_lowercase();
        let matches = |x: &rustgis_engine::analysis::ToolSpec| {
            q.is_empty() || x.name.to_lowercase().contains(&q) || x.summary.to_lowercase().contains(&q) || x.id.to_lowercase().contains(&q)
        };
        let row = |app: &mut RustGisApp, ui: &mut Ui, tool: &'static rustgis_engine::analysis::ToolSpec| {
            let (r, resp) = ui.allocate_exact_size(vec2(ui.available_width(), 42.0), Sense::click());
            resp.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, true, tl!(tool.name)));
            if resp.hovered() {
                ui.painter().rect_filled(r, CornerRadius::same(6), t.hover);
            }
            icons::paint(
                ui.painter(),
                Rect::from_center_size(pos2(r.min.x + 16.0, r.center().y), vec2(20.0, 20.0)),
                tool_icon(tool.id),
                t.icon,
                t.accent,
            );
            ui.painter().text(pos2(r.min.x + 34.0, r.min.y + 13.0), Align2::LEFT_CENTER, tl!(tool.name), semibold(12.5), t.accent_text);
            let summary: String = tl!(tool.summary).chars().take(52).collect();
            ui.painter().text(pos2(r.min.x + 34.0, r.min.y + 29.0), Align2::LEFT_CENTER, summary, regular(11.0), t.text_dim);
            if resp.on_hover_text(tl!(tool.summary)).clicked() {
                app.gp.open(Some(tool.id), &app.session);
            }
        };
        // The GeoLibre suite: a thousand tools, so its toolboxes stay closed until opened (or,
        // without the runner, a card says where to get it). Above the built-in tools when
        // browsing, so it isn't lost below them; after them when searching.
        let external = rustgis_engine::cmd::gp::geolibre_tools();
        let geolibre_section = |app: &mut RustGisApp, ui: &mut Ui| {
            if external.is_empty() {
                if q.is_empty() {
                    geolibre_card(app, ui, t);
                    ui.add_space(8.0);
                }
                return;
            }
            if q.is_empty() {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new(tl!("GeoLibre")).size(11.0).color(t.text_dim));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| geolibre_uninstall(app, ui, t));
                });
                if let Some(e) = &app.gp.uninstall_error {
                    ui.label(egui::RichText::new(e).size(11.0).color(t.red));
                }
                // As in GeoLibre's Processing menu: category › subcategory › tools
                // (`GeoLibre › Terrain › Derivatives`; a tool without a subcategory sits
                // directly in its category).
                let level = |tool: &rustgis_engine::analysis::ToolSpec, i: usize| tool.toolbox.split(" › ").nth(i);
                let mut tops: Vec<&str> = external.iter().filter_map(|x| level(x, 1)).collect();
                tops.sort();
                tops.dedup();
                for top in tops {
                    let in_top: Vec<_> = external.iter().filter(|x| level(x, 1) == Some(top)).collect();
                    egui::CollapsingHeader::new(format!("{} ({})", crate::i18n::location(top), in_top.len()))
                        .id_salt(("gp-box", top))
                        .default_open(false)
                        .show(ui, |ui| {
                            let mut subs: Vec<&str> = in_top.iter().filter_map(|x| level(x, 2)).collect();
                            subs.sort();
                            subs.dedup();
                            for sub in subs {
                                let in_sub: Vec<_> = in_top.iter().copied().filter(|x| level(x, 2) == Some(sub)).collect();
                                egui::CollapsingHeader::new(format!("{} ({})", crate::i18n::location(sub), in_sub.len()))
                                    .id_salt(("gp-box", top, sub))
                                    .default_open(false)
                                    .show(ui, |ui| {
                                        for tool in in_sub {
                                            row(app, ui, tool);
                                        }
                                    });
                            }
                            for tool in in_top.iter().copied().filter(|x| level(x, 2).is_none()) {
                                row(app, ui, tool);
                            }
                        });
                }
                ui.add_space(8.0);
            } else {
                const MAX: usize = 60;
                let found: Vec<_> = external.iter().filter(|x| matches(x)).collect();
                if !found.is_empty() {
                    ui.label(egui::RichText::new(tl!("GeoLibre")).size(11.0).color(t.text_dim));
                }
                for tool in found.iter().take(MAX) {
                    row(app, ui, tool);
                }
                if found.len() > MAX {
                    let more = crate::i18n::fmt("Refine the search to see {n} more", &[("n", &(found.len() - MAX).to_string())]);
                    ui.label(egui::RichText::new(more).size(11.0).color(t.text_dim));
                }
            }
        };
        // Plugin tools, by toolbox ("Plugins › Vector Toolkit"): first when browsing (a user
        // who just installed a plugin looks for it), with the matches when searching.
        let plugin_tools: Vec<&'static rustgis_engine::analysis::ToolSpec> =
            rustgis_engine::analysis::external_tools_of(rustgis_engine::plugins::tools::SOURCE).iter().filter(|x| matches(x)).collect();
        if !plugin_tools.is_empty() {
            ui.label(egui::RichText::new(tl!("Plugins")).size(11.0).color(t.text_dim));
            let mut boxes: Vec<&str> = plugin_tools.iter().map(|t| t.toolbox).collect();
            boxes.dedup();
            for b in boxes {
                let in_box: Vec<_> = plugin_tools.iter().filter(|x| x.toolbox == b).collect();
                egui::CollapsingHeader::new(format!("{} ({})", b.rsplit(" › ").next().unwrap_or(b), in_box.len()))
                    .id_salt(("gp-plugin", b))
                    .default_open(true)
                    .show(ui, |ui| {
                        for tool in in_box {
                            row(app, ui, tool);
                        }
                    });
            }
            ui.add_space(6.0);
        }
        if q.is_empty() {
            geolibre_section(app, ui);
        }
        let builtin = rustgis_engine::analysis::TOOLS;
        if q.is_empty() {
            // Browsing: one collapsible toolbox per top-level group, like the GeoLibre ones
            // (open: there are only a few built-in tools).
            ui.label(egui::RichText::new(tl!("RustGIS")).size(11.0).color(t.text_dim));
            let mut tops: Vec<&str> = builtin.iter().map(|t| t.toolbox.split(" › ").next().unwrap_or(t.toolbox)).collect();
            tops.sort();
            tops.dedup();
            for top in tops {
                let in_top: Vec<_> = builtin.iter().filter(|x| x.toolbox.split(" › ").next() == Some(top)).collect();
                egui::CollapsingHeader::new(format!("{} ({})", crate::i18n::location(top), in_top.len()))
                    .id_salt(("gp-builtin", top))
                    .default_open(true)
                    .show(ui, |ui| {
                        for tool in in_top {
                            row(app, ui, tool);
                        }
                    });
            }
            ui.add_space(6.0);
        } else {
            // Searching: the matches under their toolbox.
            let mut boxes: Vec<&str> = builtin.iter().map(|t| t.toolbox).collect();
            boxes.sort();
            boxes.dedup();
            for b in boxes {
                let found: Vec<_> = builtin.iter().filter(|x| x.toolbox == b && matches(x)).collect();
                if found.is_empty() {
                    continue;
                }
                ui.label(egui::RichText::new(crate::i18n::location(b)).size(11.0).color(t.text_dim));
                for tool in found {
                    row(app, ui, tool);
                }
                ui.add_space(6.0);
            }
        }
        if !q.is_empty() {
            geolibre_section(app, ui);
        }
        return;
    };
    // Tool form.
    ui.horizontal(|ui| {
        if crate::widgets::icon_button(ui, "back", tl!("Back to tools")).clicked() {
            app.gp.tool = None;
        }
        let (r, _) = ui.allocate_exact_size(vec2(22.0, 22.0), Sense::hover());
        icons::paint(ui.painter(), r, tool_icon(spec.id), t.icon, t.accent);
        ui.label(egui::RichText::new(tl!(spec.name)).font(semibold(15.0)));
    });
    ui.label(egui::RichText::new(tl!(spec.summary)).size(11.5).color(t.text_dim));
    ui.add_space(4.0);
    ui.separator();
    ui.add_space(4.0);
    // A standalone table is picked by its id, since names can repeat, and shown by its name.
    let tables: Vec<(u32, String)> = app.session.project.tables.iter().map(|t| (t.id, t.name.clone())).collect();
    let layer_names = |filter: Option<GeometryType>| -> Vec<String> {
        app.session.map().layers.iter().filter(|l| filter.is_none_or(|g| l.geometry_type() == g)).map(|l| l.name.clone()).collect()
    };
    let kind_names = |raster: bool| -> Vec<String> {
        app.session.map().layers.iter().filter(|l| l.is_raster() == raster).map(|l| l.name.clone()).collect()
    };
    let first_layer = spec
        .params
        .iter()
        .find(|p| matches!(p.kind, ParamKind::Layer | ParamKind::PolygonLayer | ParamKind::PointLayer | ParamKind::VectorLayer))
        .map(|p| p.name);
    for p in spec.params {
        let label = ui.label(egui::RichText::new(format!("{}{}", tl!(p.label), if p.required { " *" } else { "" })).size(12.0));
        if !p.help.is_empty() {
            label.on_hover_text(p.help);
        }
        let mut cur = app.gp.values.get(p.name).cloned().unwrap_or_default();
        let options: Option<Vec<String>> = match p.kind {
            // Layers by their names, then tables: the value is the name shown, and a table
            // picked is kept by its id beside it.
            ParamKind::Layer => {
                let picked = app.gp.tables.get(p.name).copied();
                let shown = if cur.is_empty() { "—" } else { cur.as_str() };
                egui::ComboBox::from_id_salt(("gp", p.name)).selected_text(shown).width(ui.available_width() - 4.0).show_ui(ui, |ui| {
                    for name in layer_names(None) {
                        if ui.selectable_label(picked.is_none() && name == cur, &name).clicked() {
                            app.gp.values.insert(p.name, name);
                            app.gp.tables.remove(p.name);
                        }
                    }
                    for (id, name) in &tables {
                        if ui.selectable_label(picked == Some(*id), name).clicked() {
                            app.gp.values.insert(p.name, name.clone());
                            app.gp.tables.insert(p.name, *id);
                        }
                    }
                });
                ui.add_space(4.0);
                continue;
            }
            ParamKind::PolygonLayer => Some(layer_names(Some(GeometryType::Polygon))),
            ParamKind::PointLayer => Some(layer_names(Some(GeometryType::Point))),
            ParamKind::Field | ParamKind::NumericField | ParamKind::CoordinateField => {
                let lname = first_layer.and_then(|n| app.gp.values.get(n)).cloned().unwrap_or_default();
                let table = first_layer.and_then(|n| app.gp.tables.get(n)).copied();
                // The table picked; else the layer or table of that name, as the tool finds it.
                let ds = match table {
                    Some(id) => app.session.project.table(id).map(|t| &t.dataset),
                    None => app.session.map().layer_by_name(&lname).map(|l| &l.dataset).or_else(|| {
                        let tables = &app.session.project.tables;
                        rustgis_engine::project::find_table(tables, &lname).and_then(|i| tables.get(i)).map(|t| &t.dataset)
                    }),
                };
                let offered = |d: &rustgis_engine::data::Dataset, i: usize, f: &rustgis_engine::data::Field| match p.kind {
                    ParamKind::NumericField => f.ty.is_numeric(),
                    ParamKind::CoordinateField => rustgis_engine::analysis::holds_coordinates(d, i),
                    _ => true,
                };
                let fields = |d: &rustgis_engine::data::Dataset| -> Vec<String> {
                    d.fields.iter().enumerate().filter(|(i, f)| offered(d, *i, f)).map(|(_, f)| f.name.clone()).collect()
                };
                // Telling which fields hold coordinates reads their rows, too much for every frame:
                // the list is kept until the project or the input changes.
                let coordinates = p.kind == ParamKind::CoordinateField;
                let key = (app.session.revision, lname, table);
                let kept = match &app.gp.coordinate_fields {
                    Some((of, kept)) if coordinates && *of == key => Some(kept.clone()),
                    _ => None,
                };
                let mut v: Vec<String> = match (ds, kept) {
                    (None, _) => Vec::new(),
                    (Some(_), Some(kept)) => kept,
                    (Some(d), None) => {
                        let v = fields(d);
                        if coordinates {
                            app.gp.coordinate_fields = Some((key, v.clone()));
                        }
                        v
                    }
                };
                // A field picked for another layer, which this one doesn't have: unset.
                if ds.is_some() && !cur.is_empty() && !v.iter().any(|f| f.eq_ignore_ascii_case(&cur)) {
                    app.gp.values.remove(p.name);
                    cur.clear();
                }
                if !p.required {
                    v.insert(0, String::new());
                }
                Some(v)
            }
            ParamKind::Choice(c) => Some(c.iter().map(|s| s.to_string()).collect()),
            ParamKind::RasterLayer | ParamKind::VectorLayer => {
                let mut v = kind_names(p.kind == ParamKind::RasterLayer);
                if !p.required {
                    v.insert(0, String::new());
                }
                Some(v)
            }
            ParamKind::Bool => {
                let mut on = cur == "true";
                if ui.checkbox(&mut on, "").changed() {
                    app.gp.values.insert(p.name, on.to_string());
                }
                ui.add_space(4.0);
                continue;
            }
            _ => None,
        };
        match options {
            Some(opts) => {
                let choice = matches!(p.kind, ParamKind::Choice(_));
                let shown = |v: &str| {
                    if v.is_empty() {
                        "—".to_string()
                    } else if choice {
                        tl!(v).to_string()
                    } else {
                        v.to_string()
                    }
                };
                egui::ComboBox::from_id_salt(("gp", p.name)).selected_text(shown(&cur)).width(ui.available_width() - 4.0).show_ui(
                    ui,
                    |ui| {
                        for o in opts {
                            if ui.selectable_label(o == cur, shown(&o)).clicked() {
                                app.gp.values.insert(p.name, o);
                            }
                        }
                    },
                );
            }
            None => {
                let mut v = cur;
                let hint = match p.kind {
                    ParamKind::Number => tl!("number"),
                    ParamKind::Path => tl!("file path"),
                    _ => "",
                };
                if ui.add(egui::TextEdit::singleline(&mut v).hint_text(hint).desired_width(f32::INFINITY)).changed() {
                    app.gp.values.insert(p.name, v);
                }
            }
        }
        ui.add_space(4.0);
    }
    ui.add_space(8.0);
    ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
        // One browser run at a time.
        if crate::widgets::primary_button(ui, tl!("Run"), app.gp.web_run.is_none(), vec2(96.0, 30.0)).clicked() {
            let mut params = serde_json::Map::new();
            // The tools of a suite or a plugin read their inputs as text; a plugin's is handed
            // every parameter, so it gets only those it declares.
            let builtin = rustgis_engine::analysis::external_source(spec).is_none();
            let plugin = spec.id.starts_with(rustgis_engine::plugins::PREFIX);
            for p in spec.params {
                if let Some(v) = app.gp.values.get(p.name).filter(|v| !v.trim().is_empty()) {
                    let val = match (p.kind, app.gp.tables.get(p.name)) {
                        // A table goes by its id: a number, which no name is taken for. Its name
                        // goes beside it (`input_name`) for whoever reads the run afterwards; the
                        // tools read only the parameters they know.
                        (ParamKind::Layer, Some(id)) => {
                            let note = format!("{}_name", p.name);
                            if !plugin && !spec.params.iter().any(|q| q.name == note) {
                                params.insert(note, json!(v));
                            }
                            if builtin { json!(id) } else { json!(id.to_string()) }
                        }
                        (ParamKind::Number, _) => v.trim().parse::<f64>().map(|n| json!(n)).unwrap_or(json!(v)),
                        (ParamKind::Bool, _) => json!(v == "true"),
                        _ => json!(v),
                    };
                    params.insert(p.name.to_string(), val);
                }
            }
            let params = Value::Object(params);
            if rustgis_engine::cmd::gp::is_geolibre_tool(spec) && app.services.geolibre_run.is_some() {
                // The browser's runner is asynchronous: start it, and `poll_geolibre` finishes.
                match rustgis_engine::cmd::gp::external_start(&mut app.session, spec.id, &params) {
                    Ok(run) => {
                        let (id, slot) = (run.id, crate::Slot::default());
                        if let Some(start) = &app.services.geolibre_run {
                            start(run, slot.clone());
                        }
                        app.gp.web_run = Some((id, spec.id, slot));
                        app.gp.last = None;
                    }
                    Err(e) => app.gp.last = Some((false, vec![e])),
                }
            } else {
                let r = app.run("gp.run", json!({"tool": spec.id, "params": params}));
                app.gp.last = Some(match r {
                    Ok(_) => (true, app.session.project.gp_history.last().map(|h| h.messages.clone()).unwrap_or_default()),
                    Err(e) => (false, vec![e]),
                });
                app.map.invalidate();
            }
        }
    });
    if app.gp.web_run.is_some() {
        ui.add_space(8.0);
        ui.horizontal(|ui| {
            ui.spinner();
            ui.label(egui::RichText::new(tl!("Running…")).size(12.0));
        });
    }
    if let Some((ok, msgs)) = &app.gp.last {
        ui.add_space(8.0);
        let (fill, stroke, head) =
            if *ok { (t.green.gamma_multiply(0.12), t.green, "Completed") } else { (t.red.gamma_multiply(0.12), t.red, "Failed") };
        egui::Frame::NONE
            .fill(fill)
            .stroke(Stroke::new(1.0, stroke))
            .corner_radius(CornerRadius::same(6))
            .inner_margin(egui::Margin::same(8))
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.label(egui::RichText::new(tl!(head)).font(semibold(12.5)).color(stroke));
                for m in msgs {
                    ui.label(egui::RichText::new(m).size(11.5));
                }
            });
    }
}

pub fn tool_icon(id: &str) -> &'static str {
    match id {
        "buffer" => "buffer",
        "clip" => "clip",
        "erase" => "erase",
        "dissolve" => "dissolve",
        "convexHull" => "hull",
        "featureToPoint" => "centroid",
        "simplify" => "simplify",
        "selectByLocation" => "select_loc",
        "selectByAttributes" => "select_attr",
        "countPoints" => "count",
        "calculateGeometry" => "calc_geom",
        "summaryStatistics" => "stats",
        "exportFeatures" => "export_layer",
        _ => "tools",
    }
}

// ---------------------------------------------------------------------------------------------
// Symbology

fn ramp_swatch(ui: &mut Ui, ramp: &Ramp, w: f32, selected: bool, t: &Tokens) -> egui::Response {
    let (r, resp) = ui.allocate_exact_size(vec2(w, 18.0), Sense::click());
    let n = 24;
    for i in 0..n {
        let c = if ramp.is_qualitative() { ramp.pick(i * 10 / n, 10) } else { ramp.at(i as f64 / (n - 1) as f64) };
        let x0 = r.min.x + r.width() * i as f32 / n as f32;
        let x1 = r.min.x + r.width() * (i + 1) as f32 / n as f32 + 0.5;
        ui.painter().rect_filled(Rect::from_min_max(pos2(x0, r.min.y), pos2(x1, r.max.y)), 0.0, c32(c));
    }
    ui.painter().rect_stroke(
        r,
        CornerRadius::same(2),
        Stroke::new(if selected { 2.0 } else { 1.0 }, if selected { t.accent } else { t.border_strong }),
        egui::StrokeKind::Outside,
    );
    resp
}

fn symbology(app: &mut RustGisApp, ui: &mut Ui, t: &Tokens) {
    let Some(l) = app.session.active_layer.and_then(|id| app.session.map().layer(id)).cloned() else {
        ui.label(egui::RichText::new(tl!("Select a layer in the Layers pane.")).color(t.text_dim));
        return;
    };
    if l.is_raster() {
        raster_symbology(app, ui, &l, t);
        return;
    }
    let g = l.geometry_type();
    ui.label(egui::RichText::new(&l.name).font(semibold(14.0)));
    ui.add_space(6.0);
    ui.label(egui::RichText::new(tl!("Primary symbology")).size(11.5).color(t.text_dim));
    let kinds: Vec<(&str, &str)> = if g == GeometryType::Point {
        vec![
            ("single", "Single Symbol"),
            ("unique", "Unique Values"),
            ("graduated", "Graduated Colors"),
            ("proportional", "Proportional Symbols"),
        ]
    } else {
        vec![("single", "Single Symbol"), ("unique", "Unique Values"), ("graduated", "Graduated Colors")]
    };
    let cur_kind = l.renderer.kind_label();
    ui.horizontal_wrapped(|ui| {
        for (code, label) in &kinds {
            let icon = match *code {
                "single" => "single",
                "unique" => "unique",
                "graduated" => "graduated",
                _ => "proportional",
            };
            let sel = cur_kind == *label;
            let (r, resp) = ui.allocate_exact_size(vec2(68.0, 56.0), Sense::click());
            ui.painter().rect_filled(
                r,
                CornerRadius::same(6),
                if sel {
                    t.checked
                } else if resp.hovered() {
                    t.hover
                } else {
                    t.pane
                },
            );
            ui.painter().rect_stroke(
                r,
                CornerRadius::same(6),
                Stroke::new(1.0, if sel { t.accent } else { t.border }),
                egui::StrokeKind::Inside,
            );
            icons::paint(
                ui.painter(),
                Rect::from_center_size(pos2(r.center().x, r.min.y + 19.0), vec2(24.0, 24.0)),
                icon,
                t.icon,
                t.accent,
            );
            let short = tl!(label).split(' ').next().unwrap_or(label);
            ui.painter().text(pos2(r.center().x, r.max.y - 11.0), Align2::CENTER_CENTER, short, regular(11.0), t.text);
            if resp.on_hover_text(tl!(*label)).clicked() && !sel {
                let _ = app.run("layer.symbology", json!({"type": code}));
            }
        }
    });
    ui.add_space(8.0);
    let numeric: Vec<String> = l.dataset.fields.iter().filter(|f| f.ty.is_numeric()).map(|f| f.name.clone()).collect();
    let all: Vec<String> = l.dataset.fields.iter().map(|f| f.name.clone()).collect();
    match &l.renderer {
        Renderer::Single(s) => symbol_editor(app, ui, *s, g, t),
        Renderer::Unique { field, classes, ramp, .. } => {
            field_combo(app, ui, "Field", field, &all, "unique");
            ramp_picker(app, ui, ramp, t);
            ui.add_space(6.0);
            ui.label(
                egui::RichText::new(crate::i18n::fmt("{n} classes", &[("n", &classes.len().to_string())])).size(11.5).color(t.text_dim),
            );
            legend_list(ui, &l, t);
        }
        Renderer::Graduated { field, breaks, ramp, method, .. } => {
            field_combo(app, ui, "Field", field, &numeric, "graduated");
            ui.horizontal(|ui| {
                ui.label(tl!("Method"));
                egui::ComboBox::from_id_salt("sym_method").selected_text(tl!(method.label())).width(180.0).show_ui(ui, |ui| {
                    for m in ClassMethod::ALL {
                        if ui.selectable_label(m == *method, tl!(m.label())).clicked() {
                            let _ = app.run("layer.symbology", json!({"type": "graduated", "field": field, "method": format!("{m:?}"), "classes": breaks.len(), "ramp": ramp.name()}));
                        }
                    }
                });
            });
            ui.horizontal(|ui| {
                ui.label(tl!("Classes"));
                let mut k = breaks.len();
                if ui.add(egui::DragValue::new(&mut k).range(1..=12)).changed() {
                    let _ = app.run(
                        "layer.symbology",
                        json!({"type": "graduated", "field": field, "method": format!("{method:?}"), "classes": k, "ramp": ramp.name()}),
                    );
                }
            });
            ramp_picker(app, ui, ramp, t);
            ui.add_space(6.0);
            legend_list(ui, &l, t);
        }
        Renderer::Proportional { field, base, .. } => {
            field_combo(app, ui, "Field", field, &numeric, "proportional");
            symbol_editor(app, ui, *base, g, t);
            legend_list(ui, &l, t);
        }
    }
    ui.add_space(10.0);
    ui.separator();
    ui.label(egui::RichText::new(tl!("Layer")).size(11.5).color(t.text_dim));
    let mut tr = l.transparency;
    ui.horizontal(|ui| {
        ui.label(tl!("Transparency"));
        ui.spacing_mut().slider_width = 150.0;
        let r = ui.add(egui::Slider::new(&mut tr, 0.0..=100.0).suffix(" %").integer());
        if r.drag_stopped() || (r.changed() && !r.dragged()) {
            let _ = app.run("layer.transparency", json!({"value": tr}));
        }
    });
}

fn field_combo(app: &mut RustGisApp, ui: &mut Ui, label: &str, cur: &str, fields: &[String], kind: &str) {
    ui.horizontal(|ui| {
        ui.label(tl!(label));
        egui::ComboBox::from_id_salt(("sym_field", kind)).selected_text(cur).width(200.0).show_ui(ui, |ui| {
            for f in fields {
                if ui.selectable_label(f == cur, f).clicked() {
                    let _ = app.run("layer.symbology", json!({"type": kind, "field": f}));
                }
            }
        });
    });
}

fn ramp_picker(app: &mut RustGisApp, ui: &mut Ui, cur: &Ramp, t: &Tokens) {
    ui.add_space(4.0);
    ui.label(egui::RichText::new(tl!("Color scheme")).size(11.5).color(t.text_dim));
    let w = (ui.available_width() - 8.0) / 2.0;
    let grid = |app: &mut RustGisApp, ui: &mut Ui, id: &str, ramps: Vec<Ramp>| {
        egui::Grid::new(id).spacing(vec2(8.0, 6.0)).show(ui, |ui| {
            for (i, r) in ramps.iter().enumerate() {
                // Built-in names are translated; a custom ramp's is the user's own.
                let tip = if matches!(r, Ramp::Custom(_)) { r.label() } else { tl!(r.label()) };
                if ramp_swatch(ui, r, w, r == cur, t).on_hover_text(tip).clicked() {
                    let _ = app.run("layer.ramp", json!({"ramp": r.name()}));
                }
                if i % 2 == 1 {
                    ui.end_row();
                }
            }
        });
    };
    grid(app, ui, "ramps", Ramp::ALL.to_vec());
    let custom: Vec<Ramp> = app.session.project.ramps.iter().map(|r| Ramp::Custom(Box::new(r.clone()))).collect();
    if !custom.is_empty() {
        ui.add_space(4.0);
        ui.label(egui::RichText::new(tl!("Custom")).size(11.5).color(t.text_dim));
        grid(app, ui, "custom_ramps", custom);
    }
    if ui
        .small_button(tl!("Add Color Ramp File…"))
        .on_hover_text(tl!("Add color ramps to the project from a .rgisramp file, a QGIS style, a GDAL color file or a GMT palette."))
        .clicked()
    {
        let _ = app.run("ui.openItem", json!({"kind": "ramps"}));
    }
}

fn legend_list(ui: &mut Ui, l: &Layer, t: &Tokens) {
    egui::Frame::NONE.stroke(Stroke::new(1.0, t.border)).corner_radius(CornerRadius::same(6)).inner_margin(egui::Margin::same(6)).show(
        ui,
        |ui| {
            ui.set_width(ui.available_width());
            for e in l.renderer.legend().iter().take(64) {
                let (r, _) = ui.allocate_exact_size(vec2(ui.available_width(), 20.0), Sense::hover());
                patch(
                    ui.painter(),
                    Rect::from_center_size(pos2(r.min.x + 12.0, r.center().y), vec2(18.0, 14.0)),
                    &e.symbol,
                    l.geometry_type(),
                );
                ui.painter().text(pos2(r.min.x + 30.0, r.center().y), Align2::LEFT_CENTER, &e.label, regular(11.5), t.text);
            }
        },
    );
}

fn symbol_editor(app: &mut RustGisApp, ui: &mut Ui, s: Symbol, g: GeometryType, t: &Tokens) {
    egui::Frame::NONE.fill(t.pane_header).corner_radius(CornerRadius::same(8)).inner_margin(egui::Margin::same(10)).show(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.horizontal(|ui| {
            let (r, _) = ui.allocate_exact_size(vec2(44.0, 36.0), Sense::hover());
            ui.painter().rect_filled(r, CornerRadius::same(4), t.ribbon);
            patch(ui.painter(), r.shrink(6.0), &s, g);
            ui.label(egui::RichText::new(tl!("Symbol")).font(semibold(12.5)));
        });
        egui::Grid::new("symbol_grid").num_columns(2).spacing(vec2(10.0, 6.0)).show(ui, |ui| {
            if g != GeometryType::Polyline {
                ui.label(tl!("Color"));
                let mut c = c32(s.fill);
                if ui.color_edit_button_srgba(&mut c).changed() {
                    let _ = app.run("layer.color", json!({"color": rgba(c).hex()}));
                }
                ui.end_row();
            }
            ui.label(tl!(if g == GeometryType::Polyline { "Line color" } else { "Outline color" }));
            let mut c = c32(s.stroke);
            if ui.color_edit_button_srgba(&mut c).changed() {
                let key = if g == GeometryType::Polyline { "layer.color" } else { "layer.outline" };
                let _ = app.run(key, json!({"color": rgba(c).hex()}));
            }
            ui.end_row();
            ui.label(tl!(if g == GeometryType::Polyline { "Line width" } else { "Outline width" }));
            let mut w = s.stroke_width;
            if ui.add(egui::DragValue::new(&mut w).range(0.0..=12.0).speed(0.05).suffix(" pt")).changed() {
                let _ = app.run("layer.outline", json!({"width": w}));
            }
            ui.end_row();
            if g == GeometryType::Point {
                ui.label(tl!("Size"));
                let mut sz = s.size;
                if ui.add(egui::DragValue::new(&mut sz).range(1.0..=60.0).speed(0.1).suffix(" pt")).changed() {
                    let _ = app.run("layer.marker", json!({"size": sz}));
                }
                ui.end_row();
                ui.label(tl!("Shape"));
                egui::ComboBox::from_id_salt("marker_shape").selected_text(tl!(s.marker.label())).show_ui(ui, |ui| {
                    for m in Marker::ALL {
                        if ui.selectable_label(m == s.marker, tl!(m.label())).clicked() {
                            let _ = app.run("layer.marker", json!({"shape": m.label()}));
                        }
                    }
                });
                ui.end_row();
            }
        });
    });
}

// ---------------------------------------------------------------------------------------------
// History and Details

fn history(app: &mut RustGisApp, ui: &mut Ui, t: &Tokens) {
    let hist = app.session.project.gp_history.clone();
    if hist.is_empty() {
        ui.label(egui::RichText::new(tl!("Tools you run appear here.")).color(t.text_dim));
        return;
    }
    for h in hist.iter().rev() {
        egui::Frame::NONE.stroke(Stroke::new(1.0, t.border)).corner_radius(CornerRadius::same(6)).inner_margin(egui::Margin::same(8)).show(
            ui,
            |ui| {
                ui.set_width(ui.available_width());
                ui.horizontal(|ui| {
                    let (r, _) = ui.allocate_exact_size(vec2(10.0, 10.0), Sense::hover());
                    ui.painter().circle_filled(r.center(), 4.5, if h.ok { t.green } else { t.red });
                    ui.label(egui::RichText::new(tl!(&h.tool)).font(semibold(12.5)));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(egui::RichText::new(format!("{} ms", h.millis)).size(11.0).color(t.text_dim));
                    });
                });
                for m in &h.messages {
                    ui.label(egui::RichText::new(m).size(11.0).color(t.text_dim));
                }
            },
        );
        ui.add_space(4.0);
    }
    if ui.button(tl!("Clear History")).clicked() {
        let _ = app.run("gp.clearHistory", Value::Null);
    }
}

fn details(app: &mut RustGisApp, ui: &mut Ui, t: &Tokens) {
    let Some(l) = app.session.active_layer.and_then(|id| app.session.map().layer(id)) else {
        ui.label(egui::RichText::new(tl!("Select a layer in the Layers pane.")).color(t.text_dim));
        return;
    };
    let e = l.extent();
    ui.label(egui::RichText::new(&l.name).font(semibold(14.0)));
    if let Some(rl) = &l.raster {
        raster_details(ui, rl, t);
        return;
    }
    egui::Grid::new("details").num_columns(2).striped(true).spacing(vec2(12.0, 4.0)).show(ui, |ui| {
        let rows = [
            ("Geometry", tl!(l.geometry_type().label()).to_string()),
            ("Features", l.dataset.features.len().to_string()),
            ("Fields", l.dataset.fields.len().to_string()),
            ("Selected", l.selection.len().to_string()),
            ("Symbology", tl!(l.renderer.kind_label()).to_string()),
            ("Source", l.dataset.source.clone().unwrap_or_else(|| tl!("In project").into())),
            ("West", format!("{:.4}°", e.min_x)),
            ("East", format!("{:.4}°", e.max_x)),
            ("South", format!("{:.4}°", e.min_y)),
            ("North", format!("{:.4}°", e.max_y)),
            ("Definition query", l.definition_query.clone().unwrap_or_else(|| "—".into())),
        ];
        for (k, v) in rows {
            ui.label(egui::RichText::new(tl!(k)).color(t.text_dim));
            ui.label(v);
            ui.end_row();
        }
    });
    ui.add_space(8.0);
    ui.label(egui::RichText::new(tl!("Fields")).font(semibold(12.5)));
    for f in &l.dataset.fields {
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new(&f.name).monospace().size(11.5));
            ui.label(egui::RichText::new(tl!(f.ty.label())).size(11.0).color(t.text_dim));
        });
    }
}

// ---------------------------------------------------------------------------------------------
// Rasters

/// A band's display name: "Band 3" in the interface language.
pub fn band_name(name: &str) -> String {
    let n = name.trim_start_matches(|c: char| !c.is_ascii_digit());
    if !n.is_empty() && name == format!("Band {n}") { crate::i18n::fmt("Band {n}", &[("n", n)]) } else { name.to_string() }
}

/// Legend rows for a raster layer, indented by `indent`.
fn raster_legend_rows(ui: &mut Ui, entries: &[RasterLegend], indent: f32, t: &Tokens) {
    for e in entries.iter().take(40) {
        match e {
            RasterLegend::Ramp { ramp, invert, high, low } => {
                let (r, _) = ui.allocate_exact_size(vec2(ui.available_width(), 52.0), Sense::hover());
                let bar = Rect::from_min_size(pos2(r.min.x + indent + 3.0, r.min.y + 4.0), vec2(14.0, 44.0));
                let n = 22;
                for i in 0..n {
                    let tt = 1.0 - i as f64 / (n - 1) as f64;
                    let c = ramp.at(if *invert { 1.0 - tt } else { tt });
                    let y0 = bar.min.y + bar.height() * i as f32 / n as f32;
                    let y1 = bar.min.y + bar.height() * (i + 1) as f32 / n as f32 + 0.5;
                    ui.painter().rect_filled(Rect::from_min_max(pos2(bar.min.x, y0), pos2(bar.max.x, y1)), 0.0, c32(c));
                }
                ui.painter().rect_stroke(bar, CornerRadius::same(2), Stroke::new(1.0, t.border_strong), egui::StrokeKind::Outside);
                let shown = |v: &str| match v {
                    "Lit" => tl!("Lit").to_string(),
                    "Shaded" => tl!("Shaded").to_string(),
                    other => other.to_string(),
                };
                ui.painter().text(pos2(bar.max.x + 8.0, bar.min.y + 4.0), Align2::LEFT_CENTER, shown(high), regular(11.5), t.text);
                ui.painter().text(pos2(bar.max.x + 8.0, bar.max.y - 4.0), Align2::LEFT_CENTER, shown(low), regular(11.5), t.text);
            }
            RasterLegend::Class { color, label } | RasterLegend::Channel { color, label } => {
                let (r, _) = ui.allocate_exact_size(vec2(ui.available_width(), 20.0), Sense::hover());
                let sw = Rect::from_center_size(pos2(r.min.x + indent + 10.0, r.center().y), vec2(16.0, 12.0));
                ui.painter().rect_filled(sw, CornerRadius::same(2), c32(*color));
                ui.painter().rect_stroke(sw, CornerRadius::same(2), Stroke::new(1.0, t.border_strong), egui::StrokeKind::Inside);
                let shown = match e {
                    RasterLegend::Channel { color, label } => {
                        let ch = if color.r() > color.g() && color.r() > color.b() {
                            tl!("Red")
                        } else if color.g() > color.b() {
                            tl!("Green")
                        } else {
                            tl!("Blue")
                        };
                        format!("{ch}: {}", band_name(label))
                    }
                    _ => label.clone(),
                };
                ui.painter().text(pos2(sw.max.x + 8.0, r.center().y), Align2::LEFT_CENTER, shown, regular(11.5), t.text);
            }
        }
    }
}

fn raster_details(ui: &mut Ui, rl: &rustgis_engine::map::RasterLayer, t: &Tokens) {
    let Some(d) = &rl.data else {
        ui.label(egui::RichText::new(tl!("The raster's data couldn't be read.")).color(t.red));
        if let Some(p) = &rl.path {
            ui.label(egui::RichText::new(p).monospace().size(11.0));
        }
        return;
    };
    let e = d.extent();
    let (cx, cy) = d.cell_size();
    let unit = if d.crs.is_geographic() { "°" } else { " m" };
    let mut rows = vec![
        ("Size", format!("{} × {}", d.width, d.height)),
        ("Bands", d.band_count().to_string()),
        ("Pixel type", d.data_type.clone()),
        ("Cell size", format!("{} × {}{unit}", rustgis_engine::map::symbology::fmt_num(cx), rustgis_engine::map::symbology::fmt_num(cy))),
        ("Coordinate system", d.crs.label()),
        ("NoData", d.nodata.map(|v| v.to_string()).unwrap_or_else(|| "—".into())),
        ("Symbology", tl!(rl.style.renderer.label()).to_string()),
        ("Stretch", tl!(rl.style.stretch.label()).to_string()),
        ("Source", rl.path.clone().or(rl.file_name.clone()).unwrap_or_else(|| tl!("In project").into())),
        ("West", format!("{:.4}°", e.min_x)),
        ("East", format!("{:.4}°", e.max_x)),
        ("South", format!("{:.4}°", e.min_y)),
        ("North", format!("{:.4}°", e.max_y)),
    ];
    if let Some((w, h)) = d.overview {
        rows.insert(1, ("Full size", format!("{w} × {h}")));
    }
    let value_w = (ui.available_width() - 140.0).max(80.0);
    egui::Grid::new("raster_details").num_columns(2).striped(true).max_col_width(value_w).spacing(vec2(12.0, 4.0)).show(ui, |ui| {
        for (k, v) in rows {
            ui.label(egui::RichText::new(tl!(k)).color(t.text_dim));
            // Long paths are cut short instead of widening the pane; hover shows them whole.
            ui.add(egui::Label::new(&v).truncate()).on_hover_text(&v);
            ui.end_row();
        }
    });
    ui.add_space(8.0);
    ui.label(egui::RichText::new(tl!("Statistics")).font(semibold(12.5)));
    let f = rustgis_engine::map::symbology::fmt_num;
    egui::Grid::new("raster_stats").num_columns(5).striped(true).spacing(vec2(10.0, 4.0)).show(ui, |ui| {
        for h in ["Band", "Min", "Max", "Mean", "Std. Dev."] {
            ui.label(egui::RichText::new(tl!(h)).size(11.0).color(t.text_dim));
        }
        ui.end_row();
        for b in d.bands.iter().take(64) {
            ui.label(egui::RichText::new(band_name(&b.name)).size(11.5));
            for v in [b.stats.min, b.stats.max, b.stats.mean, b.stats.std_dev] {
                ui.label(egui::RichText::new(f(v)).size(11.5).monospace());
            }
            ui.end_row();
        }
    });
}

fn raster_symbology(app: &mut RustGisApp, ui: &mut Ui, l: &Layer, t: &Tokens) {
    ui.label(egui::RichText::new(&l.name).font(semibold(14.0)));
    let Some(rl) = l.raster.clone() else { return };
    let Some(d) = rl.data.clone() else {
        ui.label(egui::RichText::new(tl!("The raster's data couldn't be read.")).color(t.red));
        return;
    };
    let style = rl.style.clone();
    ui.add_space(6.0);
    ui.label(egui::RichText::new(tl!("Primary symbology")).size(11.5).color(t.text_dim));
    let has_categories = d.bands.first().is_some_and(|b| b.stats.categories.is_some());
    let palette = matches!(d.color, rustgis_engine::data::raster::ColorHint::Palette(_));
    ui.horizontal_wrapped(|ui| {
        for (code, label) in RasterRenderer::KINDS {
            let ok = match code {
                "rgb" => d.band_count() >= 3,
                "unique" => has_categories,
                "colormap" => palette,
                _ => true,
            };
            let icon = match code {
                "stretch" => "stretch",
                "rgb" => "rgb",
                "classify" => "classify",
                "unique" => "unique",
                "colormap" => "raster_layer",
                _ => "relief",
            };
            let sel = style.renderer.code() == code;
            let (r, resp) = ui.allocate_exact_size(vec2(68.0, 56.0), if ok { Sense::click() } else { Sense::hover() });
            let fill = if sel {
                t.checked
            } else if resp.hovered() && ok {
                t.hover
            } else {
                t.pane
            };
            ui.painter().rect_filled(r, CornerRadius::same(6), fill);
            ui.painter().rect_stroke(
                r,
                CornerRadius::same(6),
                Stroke::new(1.0, if sel { t.accent } else { t.border }),
                egui::StrokeKind::Inside,
            );
            let ink = if ok { t.icon } else { t.text_disabled };
            icons::paint(ui.painter(), Rect::from_center_size(pos2(r.center().x, r.min.y + 19.0), vec2(24.0, 24.0)), icon, ink, t.accent);
            let shown = tl!(label);
            let short = if shown.chars().count() > 10 { shown.split(' ').next().unwrap_or(shown) } else { shown };
            ui.painter().text(
                pos2(r.center().x, r.max.y - 11.0),
                Align2::CENTER_CENTER,
                short,
                regular(11.0),
                if ok { t.text } else { t.text_disabled },
            );
            if resp.on_hover_text(shown).clicked() && ok && !sel {
                let _ = app.run("raster.symbology", json!({"type": code}));
            }
        }
    });
    ui.add_space(8.0);
    let bands: Vec<String> = d.bands.iter().map(|b| b.name.clone()).collect();
    let band_combo = |app: &mut RustGisApp, ui: &mut Ui, label: &str, cur: usize, run: &dyn Fn(usize) -> (String, Value)| {
        ui.horizontal(|ui| {
            ui.label(tl!(label));
            egui::ComboBox::from_id_salt(("raster_band", label))
                .selected_text(band_name(bands.get(cur).map(String::as_str).unwrap_or("")))
                .width(170.0)
                .show_ui(ui, |ui| {
                    for (i, b) in bands.iter().enumerate() {
                        if ui.selectable_label(i == cur, band_name(b)).clicked() {
                            let (id, p) = run(i);
                            let _ = app.run(&id, p);
                        }
                    }
                });
        });
    };
    match &style.renderer {
        RasterRenderer::Stretch { band, ramp, invert } => {
            let code = style.renderer.code();
            band_combo(app, ui, "Band", *band, &|i| ("raster.symbology".into(), json!({"type": code, "band": i + 1})));
            ramp_picker(app, ui, ramp, t);
            let mut inv = *invert;
            if ui.checkbox(&mut inv, tl!("Invert color scheme")).changed() {
                let _ = app.run("raster.symbology", json!({"type": "stretch", "invert": inv, "ramp": ramp.name()}));
            }
        }
        RasterRenderer::Rgb { red, green, blue } => {
            for (label, key, cur) in [("Red", "red", *red), ("Green", "green", *green), ("Blue", "blue", *blue)] {
                band_combo(app, ui, label, cur, &|i| ("raster.bands".into(), json!({ key: i + 1 })));
            }
        }
        RasterRenderer::Classify { band, method, ramp, breaks } => {
            let (m, k, r) = (*method, breaks.len(), ramp.name().to_string());
            band_combo(app, ui, "Band", *band, &|i| {
                ("raster.symbology".into(), json!({"type": "classify", "band": i + 1, "classes": k, "method": format!("{m:?}")}))
            });
            ui.horizontal(|ui| {
                ui.label(tl!("Method"));
                egui::ComboBox::from_id_salt("raster_method").selected_text(tl!(method.label())).width(180.0).show_ui(ui, |ui| {
                    for mm in ClassMethod::ALL {
                        if ui.selectable_label(mm == m, tl!(mm.label())).clicked() {
                            let _ = app
                                .run("raster.symbology", json!({"type": "classify", "method": format!("{mm:?}"), "classes": k, "ramp": r}));
                        }
                    }
                });
            });
            ui.horizontal(|ui| {
                ui.label(tl!("Classes"));
                let mut kk = k;
                if ui.add(egui::DragValue::new(&mut kk).range(1..=32)).changed() {
                    let _ = app.run("raster.symbology", json!({"type": "classify", "method": format!("{m:?}"), "classes": kk, "ramp": r}));
                }
            });
            ramp_picker(app, ui, ramp, t);
        }
        RasterRenderer::UniqueValues { band, ramp, values } => {
            band_combo(app, ui, "Band", *band, &|i| ("raster.symbology".into(), json!({"type": "unique", "band": i + 1})));
            ramp_picker(app, ui, ramp, t);
            ui.label(
                egui::RichText::new(crate::i18n::fmt("{n} classes", &[("n", &values.len().to_string())])).size(11.5).color(t.text_dim),
            );
        }
        RasterRenderer::Colormap => {
            ui.label(egui::RichText::new(tl!("Colors from the raster's own color table.")).size(11.5).color(t.text_dim));
        }
        RasterRenderer::ShadedRelief { band, ramp, azimuth, altitude, z_factor } => {
            let (az, alt, zf, rp) = (*azimuth, *altitude, *z_factor, ramp.as_ref().map(|r| r.name().to_string()));
            let relief = move |band: usize, az: f64, alt: f64, zf: f64| {
                let mut p = json!({"type": "shadedRelief", "band": band + 1, "azimuth": az, "altitude": alt, "zFactor": zf});
                match &rp {
                    Some(r) => p["ramp"] = json!(r),
                    None => p["grey"] = json!(true),
                }
                p
            };
            band_combo(app, ui, "Band", *band, &|i| ("raster.symbology".into(), relief(i, az, alt, zf)));
            egui::Grid::new("relief").num_columns(2).spacing(vec2(10.0, 4.0)).show(ui, |ui| {
                for (label, v, range, speed, which) in
                    [("Azimuth", az, 0.0..=360.0, 1.0, 0), ("Altitude", alt, 0.0..=90.0, 0.5, 1), ("Z factor", zf, 0.001..=1000.0, 0.01, 2)]
                {
                    ui.label(tl!(label));
                    let mut x = v;
                    let r = ui.add(egui::DragValue::new(&mut x).range(range).speed(speed).max_decimals(3));
                    if r.drag_stopped() || (r.changed() && !r.dragged()) {
                        let (a, b, c) = match which {
                            0 => (x, alt, zf),
                            1 => (az, x, zf),
                            _ => (az, alt, x),
                        };
                        let _ = app.run("raster.symbology", relief(*band, a, b, c));
                    }
                    ui.end_row();
                }
            });
            let mut tint = ramp.is_some();
            if ui.checkbox(&mut tint, tl!("Tint by elevation")).changed() {
                let mut p = relief(*band, az, alt, zf);
                if tint {
                    p["ramp"] = json!("Terrain");
                    p["grey"] = json!(false);
                } else {
                    p["grey"] = json!(true);
                }
                let _ = app.run("raster.symbology", p);
            }
            if let Some(r) = ramp {
                ramp_picker(app, ui, r, t);
            }
        }
    }
    if !matches!(style.renderer, RasterRenderer::Colormap | RasterRenderer::UniqueValues { .. } | RasterRenderer::Classify { .. }) {
        ui.add_space(6.0);
        ui.horizontal(|ui| {
            ui.label(tl!("Stretch type"));
            egui::ComboBox::from_id_salt("raster_stretch").selected_text(tl!(style.stretch.label())).width(170.0).show_ui(ui, |ui| {
                for (code, label) in Stretch::KINDS {
                    if ui.selectable_label(style.stretch.code() == code, tl!(label)).clicked() {
                        let _ = app.run("raster.stretch", json!({"type": code}));
                    }
                }
            });
        });
        match style.stretch {
            Stretch::PercentClip { low, high } => {
                ui.horizontal(|ui| {
                    let (mut lo, mut hi) = (low, high);
                    ui.label(tl!("Min %"));
                    let a = ui.add(egui::DragValue::new(&mut lo).range(0.0..=49.0).speed(0.1).max_decimals(2));
                    ui.label(tl!("Max %"));
                    let b = ui.add(egui::DragValue::new(&mut hi).range(0.0..=49.0).speed(0.1).max_decimals(2));
                    if a.drag_stopped() || b.drag_stopped() || ((a.changed() || b.changed()) && !a.dragged() && !b.dragged()) {
                        let _ = app.run("raster.stretch", json!({"type": "percentClip", "low": lo, "high": hi}));
                    }
                });
            }
            Stretch::StandardDeviation { n } => {
                ui.horizontal(|ui| {
                    let mut nn = n;
                    ui.label(tl!("Deviations"));
                    let a = ui.add(egui::DragValue::new(&mut nn).range(0.1..=10.0).speed(0.05).max_decimals(2));
                    if a.drag_stopped() || (a.changed() && !a.dragged()) {
                        let _ = app.run("raster.stretch", json!({"type": "standardDeviation", "n": nn}));
                    }
                });
            }
            _ => {}
        }
        let mut dra = style.dra;
        if ui.checkbox(&mut dra, tl!("Dynamic range adjustment (stretch the pixels in view)")).changed() {
            let _ = app.run("raster.dra", json!({"on": dra}));
        }
    }
    ui.horizontal(|ui| {
        ui.label(tl!("Resampling"));
        egui::ComboBox::from_id_salt("raster_resampling").selected_text(tl!(style.resampling.label())).width(170.0).show_ui(ui, |ui| {
            for r in Resampling::ALL {
                if ui.selectable_label(r == style.resampling, tl!(r.label())).clicked() {
                    let _ = app.run("raster.resampling", json!({"type": format!("{r:?}")}));
                }
            }
        });
    });
    ui.add_space(6.0);
    egui::Frame::NONE.stroke(Stroke::new(1.0, t.border)).corner_radius(CornerRadius::same(6)).inner_margin(egui::Margin::same(6)).show(
        ui,
        |ui| {
            ui.set_width(ui.available_width());
            raster_legend_rows(ui, &rl.legend(), 2.0, t);
        },
    );
    ui.add_space(10.0);
    ui.separator();
    ui.label(egui::RichText::new(tl!("Layer")).size(11.5).color(t.text_dim));
    let mut tr = l.transparency;
    ui.horizontal(|ui| {
        ui.label(tl!("Transparency"));
        ui.spacing_mut().slider_width = 150.0;
        let r = ui.add(egui::Slider::new(&mut tr, 0.0..=100.0).suffix(" %").integer());
        if r.drag_stopped() || (r.changed() && !r.dragged()) {
            let _ = app.run("layer.transparency", json!({"value": tr}));
        }
    });
}
