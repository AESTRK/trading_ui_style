//! Barre d'outils standard — bouton Config Manager (deep link app).

use eframe::egui::{
    self, CornerRadius, FontId, RichText, Sense, Stroke, StrokeKind, Ui, Vec2, Widget,
};

use crate::config_window::{self, config_manager_toolbar_button, TOOLBAR_CONTROL_HEIGHT};
use crate::TEXT_SIZES;

/// Hauteur max du panneau toolbar (wrap ~2 lignes — pas de `ScrollArea` horizontale).
/// Au-delà, egui réserve mal la hauteur → bande noire (viewport) sous la toolbar.
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
            out = Some(ui.horizontal_wrapped(|ui| {
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
            }).inner);
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

/// Libellé menu toolbar (texte seul — la flèche est peinte, pas un glyphe Unicode).
pub fn toolbar_menu_button_label(label: &str) -> String {
    label.to_string()
}

const TOOLBAR_MENU_CHEVRON_WIDTH: f32 = 10.0;
const TOOLBAR_MENU_CHEVRON_GAP: f32 = 8.0;

/// Bouton menu : zone texte à gauche, chevron réservé à droite (pas de recouvrement).
struct ToolbarMenuButton<'a> {
    label: &'a str,
    menu_open: bool,
}

impl Widget for ToolbarMenuButton<'_> {
    fn ui(self, ui: &mut Ui) -> egui::Response {
        let font = toolbar_font();
        let padding = ui.spacing().button_padding;
        let text_color = ui.visuals().text_color();
        let galley = ui.painter().layout(
            self.label.to_owned(),
            font.clone(),
            text_color,
            f32::INFINITY,
        );
        let text_w = galley.size().x;
        let width =
            padding.x + text_w + TOOLBAR_MENU_CHEVRON_GAP + TOOLBAR_MENU_CHEVRON_WIDTH + padding.x;
        let height = TOOLBAR_CONTROL_HEIGHT.max(ui.spacing().interact_size.y);
        let desired = Vec2::new(width, height);

        let (rect, response) = ui.allocate_exact_size(desired, Sense::click());

        if ui.is_rect_visible(rect) {
            let visuals = ui.style().interact(&response);
            let (bg_fill, bg_stroke) = if self.menu_open {
                let open = &ui.visuals().widgets.open;
                (open.weak_bg_fill, open.bg_stroke)
            } else {
                (visuals.bg_fill, visuals.bg_stroke)
            };
            ui.painter().rect(
                rect,
                CornerRadius::same(4),
                bg_fill,
                bg_stroke,
                StrokeKind::Inside,
            );

            let inner = rect.shrink2(Vec2::new(padding.x, 0.0));
            let chevron_left = inner.right() - TOOLBAR_MENU_CHEVRON_WIDTH;
            let label_rect = egui::Rect::from_min_max(
                inner.min,
                egui::pos2(chevron_left - TOOLBAR_MENU_CHEVRON_GAP, inner.max.y),
            );
            ui.painter().with_clip_rect(label_rect).text(
                label_rect.left_center(),
                egui::Align2::LEFT_CENTER,
                self.label,
                font,
                visuals.text_color(),
            );
            let chevron = egui::Rect::from_min_max(
                egui::pos2(chevron_left, rect.top()),
                egui::pos2(inner.right(), rect.bottom()),
            );
            crate::widgets::draw_triangle(ui, chevron, true, visuals.text_color());
        }

        response.on_hover_cursor(egui::CursorIcon::PointingHand)
    }
}

/// Menu déroulant toolbar — même style bouton que [`config_manager_toolbar_button`].
pub fn toolbar_menu_button<R>(
    ui: &mut egui::Ui,
    label: &str,
    add_contents: impl FnOnce(&mut egui::Ui) -> R,
) -> egui::InnerResponse<Option<R>> {
    let bar_id = ui.id();
    let mut bar_state = egui::menu::BarState::load(ui.ctx(), bar_id);
    let button_id = ui.make_persistent_id(("toolbar_menu", label));
    let menu_open = bar_state
        .as_ref()
        .is_some_and(|root| root.id == button_id);

    let button_response = ui
        .push_id(button_id, |ui| ui.add(ToolbarMenuButton { label, menu_open }))
        .inner;

    let inner = bar_state.bar_menu(&button_response, add_contents);
    bar_state.store(ui.ctx(), bar_id);
    egui::InnerResponse::new(inner.map(|r| r.inner), button_response)
}

/// Séparateur avant le contenu métier app (onglets, filtres…) — pas entre boutons stack.
pub fn toolbar_before_app_content(ui: &mut egui::Ui) {
    toolbar_separator(ui);
}
