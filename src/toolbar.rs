//! Barre d'outils standard — bouton Config Manager (deep link app).

use eframe::egui::{self, FontId, RichText, Stroke};

use crate::config_window::{self, config_manager_toolbar_button, TOOLBAR_CONTROL_HEIGHT};
use crate::TEXT_SIZES;

/// Hauteur max du panneau toolbar (wrap ~2 lignes status + onglets).
/// Au-delà, egui réserve mal la hauteur → bande noire (viewport) sous la toolbar.
/// ~2 lignes si le contenu app reste dans la toolbar (préférer bandeau aux / header pour le métier lourd).
const TOOLBAR_PANEL_MAX_HEIGHT: f32 = 72.0;
/// Marge toolbar (compacte).
const TOOLBAR_PANEL_INNER_MARGIN: i8 = 2;

/// Séparateur vertical compact — ne pas utiliser `ui.separator()` dans la toolbar horizontale.
pub fn toolbar_separator(ui: &mut egui::Ui) {
    let h = TOOLBAR_CONTROL_HEIGHT;
    let (rect, _) = ui.allocate_exact_size(egui::vec2(6.0, h), egui::Sense::hover());
    let stroke = ui.visuals().widgets.noninteractive.bg_stroke;
    ui.painter().vline(
        rect.center().x,
        rect.top()..=rect.bottom(),
        stroke,
    );
}

/// Panneau top compact : Config Manager, menu **Export**, puis contenu app.
pub fn show_app_toolbar<R>(
    ctx: &egui::Context,
    crate_app_id: &str,
    add_contents: impl FnOnce(&mut egui::Ui) -> R,
) -> R {
    show_app_toolbar_ex(ctx, crate_app_id, None, None, add_contents)
}

/// Toolbar avec menu **Rafraîchir** uniquement.
pub fn show_app_toolbar_with_refresh<R>(
    ctx: &egui::Context,
    crate_app_id: &str,
    refresh: Option<&crate::stack_refresh_menu::RefreshToolbarHook>,
    add_contents: impl FnOnce(&mut egui::Ui) -> R,
) -> R {
    show_app_toolbar_ex(ctx, crate_app_id, None, refresh, add_contents)
}

/// Comme [`show_app_toolbar`] avec export métier **CSV/Excel** (`ExportToolbarHook::spreadsheet`).
pub fn show_app_toolbar_ex<R>(
    ctx: &egui::Context,
    crate_app_id: &str,
    export: Option<&crate::stack_export_menu::ExportToolbarHook>,
    refresh: Option<&crate::stack_refresh_menu::RefreshToolbarHook>,
    add_contents: impl FnOnce(&mut egui::Ui) -> R,
) -> R {
    crate::stack_export_menu::poll_stack_export_menu(ctx, crate_app_id);
    let mut out = None;
    let panel_id = egui::Id::new(("app_toolbar_v2", crate_app_id));
    let toolbar_frame = crate::egui_theme::stack_top_panel_frame(ctx).inner_margin(
        egui::Margin::symmetric(4, TOOLBAR_PANEL_INNER_MARGIN),
    );
    egui::TopBottomPanel::top(panel_id)
        .resizable(false)
        .show_separator_line(false)
        .frame(toolbar_frame)
        .max_height(TOOLBAR_PANEL_MAX_HEIGHT)
        .show(ctx, |ui| {
            crate::egui_theme::paint_top_stack_panel_bg(ui);
            let lane_h = TOOLBAR_CONTROL_HEIGHT + 6.0;
            out = Some(
                egui::ScrollArea::horizontal()
                    .id_salt(("app_toolbar_lane", crate_app_id))
                    .drag_to_scroll(true)
                    .max_height(lane_h)
                    .show(ui, |ui| {
                        ui.set_max_height(lane_h);
                        ui.horizontal(|ui| {
                            apply_toolbar_row_style(ui);
                            config_manager_toolbar_button(ui, crate_app_id);
                            let export_shown = crate::stack_export_menu::draw_export_menu(
                                ui,
                                ctx,
                                crate_app_id,
                                export,
                            );
                            let _refresh_shown = crate::stack_refresh_menu::draw_refresh_menu(
                                ui,
                                ctx,
                                crate_app_id,
                                refresh,
                            );
                            let _ = export_shown;
                            add_contents(ui)
                        })
                        .inner
                    })
                    .inner,
            );
        });
    out.expect("toolbar panel")
}

/// Libellé toolbar (ex. « Venue IPC ») — même hauteur que le bouton Config Manager.
pub fn toolbar_row_caption(ui: &mut egui::Ui, text: impl Into<RichText>) {
    let text = text.into();
    ui.allocate_ui_with_layout(
        egui::vec2(0.0, TOOLBAR_CONTROL_HEIGHT),
        egui::Layout::left_to_right(egui::Align::Center),
        |ui| {
            ui.label(text);
        },
    );
}

/// Champ lecture seule — même widget / hauteur que `config_manager_toolbar_button`.
pub fn toolbar_readonly_chip(
    ui: &mut egui::Ui,
    width: f32,
    label: RichText,
    fill: egui::Color32,
    stroke: egui::Color32,
) -> egui::Response {
    ui.add_enabled(
        false,
        egui::Button::new(label)
            .fill(fill)
            .stroke(Stroke::new(1.0_f32, stroke))
            .corner_radius(4.0)
            .min_size(egui::vec2(width, TOOLBAR_CONTROL_HEIGHT)),
    )
}

/// Statut toolbar (point + texte) dans la hauteur standard.
pub fn toolbar_row_status(
    ui: &mut egui::Ui,
    dot_color: egui::Color32,
    text: RichText,
    draw_dot: impl FnOnce(&mut egui::Ui, egui::Rect, egui::Color32),
) {
    ui.allocate_ui_with_layout(
        egui::vec2(0.0, TOOLBAR_CONTROL_HEIGHT),
        egui::Layout::left_to_right(egui::Align::Center),
        |ui| {
            ui.spacing_mut().item_spacing.x = 4.0;
            let (rect, _) = ui.allocate_exact_size(egui::vec2(8.0, 8.0), egui::Sense::hover());
            draw_dot(ui, rect, dot_color);
            ui.label(text);
        },
    );
}

/// Police toolbar standard (12 px proportional).
pub fn toolbar_font() -> FontId {
    FontId::proportional(TEXT_SIZES.toolbar)
}

/// Hauteur d’interaction uniforme pour la ligne toolbar (boutons, menus, onglets).
pub fn apply_toolbar_row_style(ui: &mut egui::Ui) {
    ui.spacing_mut().item_spacing = egui::vec2(8.0, 4.0);
    let h = TOOLBAR_CONTROL_HEIGHT;
    let is = &mut ui.spacing_mut().interact_size;
    is.y = is.y.max(h);
}

/// Bouton texte toolbar (ex. « Limites API », OK chartboard).
pub fn toolbar_text_button(ui: &mut egui::Ui, label: &str) -> egui::Response {
    ui.add(config_window::toolbar_styled_button(
        RichText::new(label).font(toolbar_font()),
    ))
}

/// `selectable_label` centré sur la hauteur toolbar.
pub fn toolbar_selectable_label(
    ui: &mut egui::Ui,
    selected: bool,
    text: impl Into<RichText>,
) -> egui::Response {
    let text = text.into();
    ui.allocate_ui_with_layout(
        egui::vec2(0.0, TOOLBAR_CONTROL_HEIGHT),
        egui::Layout::left_to_right(egui::Align::Center),
        |ui| ui.selectable_label(selected, text),
    )
    .inner
}

/// Libellé menu toolbar — même gabarit que Config Manager + indicateur déroulant.
pub fn toolbar_menu_button_label(label: &str) -> String {
    format!("{label}  ▾")
}

/// Menu déroulant toolbar — même style bouton que [`config_manager_toolbar_button`].
pub fn toolbar_menu_button<R>(
    ui: &mut egui::Ui,
    label: &str,
    add_contents: impl FnOnce(&mut egui::Ui) -> R,
) -> egui::InnerResponse<Option<R>> {
    let button = config_window::toolbar_styled_button(
        RichText::new(toolbar_menu_button_label(label)).font(toolbar_font()),
    );
    egui::menu::menu_custom_button(ui, button, add_contents)
}

/// Séparateur avant le contenu métier app (onglets, filtres…) — pas entre boutons stack.
pub fn toolbar_before_app_content(ui: &mut egui::Ui) {
    toolbar_separator(ui);
}
