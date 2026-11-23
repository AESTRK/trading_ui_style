//! Onglets workspace (ex. Graph | Catalogue) — texte + souligné, pas des pseudo-boutons `selectable_value`.

use eframe::egui::{self, Color32, FontId, RichText, Sense, Stroke, Ui};

use crate::config_window::TOOLBAR_CONTROL_HEIGHT;
use crate::egui_theme::{color, theme_mode};
use crate::toolbar::toolbar_font;
use crate::{palette, TEXT_SIZES};

#[derive(Debug, Clone)]
pub struct WorkspaceTabStyle {
    pub active_text: Color32,
    pub inactive_text: Color32,
    pub underline: Color32,
    pub item_spacing: f32,
    pub font: FontId,
    /// Variante barre du haut : souligné dans la hauteur toolbar, espacement réduit.
    pub in_toolbar: bool,
}

impl Default for WorkspaceTabStyle {
    fn default() -> Self {
        Self {
            active_text: Color32::WHITE,
            inactive_text: Color32::GRAY,
            underline: Color32::WHITE,
            item_spacing: 20.0,
            font: FontId::proportional(TEXT_SIZES.table),
            in_toolbar: false,
        }
    }
}

/// Style aligné sur le thème courant (taille table, comme Klines Cache).
pub fn workspace_tab_style_from_ui(ui: &Ui) -> WorkspaceTabStyle {
    let p = palette(theme_mode(ui.ctx()));
    WorkspaceTabStyle {
        active_text: color(p.text),
        inactive_text: color(p.muted),
        underline: color(p.text),
        item_spacing: 20.0,
        font: FontId::proportional(TEXT_SIZES.table),
        in_toolbar: false,
    }
}

/// Onglets dans `show_app_toolbar` (alignés Config Manager, max ~72 px).
pub fn workspace_tab_style_for_toolbar(ui: &Ui) -> WorkspaceTabStyle {
    let p = palette(theme_mode(ui.ctx()));
    WorkspaceTabStyle {
        active_text: color(p.text),
        inactive_text: color(p.muted),
        underline: color(p.text),
        item_spacing: 12.0,
        font: toolbar_font(),
        in_toolbar: true,
    }
}

pub struct WorkspaceTab<'a> {
    pub id: &'a str,
    pub label: &'a str,
    pub hint: Option<&'a str>,
}

/// Bande d’onglets horizontale. Retourne l’`id` cliqué (même si déjà actif).
pub fn show_workspace_tab_strip<'a>(
    ui: &mut Ui,
    style: &WorkspaceTabStyle,
    tabs: &'a [WorkspaceTab<'a>],
    active_id: &str,
) -> Option<&'a str> {
    let mut picked: Option<&'a str> = None;
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = style.item_spacing;
        for tab in tabs {
            let active = tab.id == active_id;
            let rt = if active {
                RichText::new(tab.label)
                    .strong()
                    .font(style.font.clone())
                    .color(style.active_text)
            } else {
                RichText::new(tab.label)
                    .font(style.font.clone())
                    .color(style.inactive_text)
            };
            let mut resp = ui
                .add(egui::Label::new(rt).sense(Sense::click()))
                .on_hover_cursor(egui::CursorIcon::PointingHand);
            if let Some(h) = tab.hint {
                resp = resp.on_hover_text(h);
            }
            if resp.clicked() {
                picked = Some(tab.id);
            }
            if active {
                let r = resp.rect;
                let y = if style.in_toolbar {
                    r.bottom() - 1.0
                } else {
                    r.bottom() + 5.0
                };
                ui.painter().hline(
                    r.left() + 2.0..=r.right() - 2.0,
                    y,
                    Stroke::new(2.0_f32, style.underline),
                );
            }
        }
    });
    if !style.in_toolbar {
        // Le souligné est peint hors du `rect` du label — réserver l’air avant le séparateur.
        ui.add_space(4.0);
    }
    picked
}

/// Espace + séparateur sous une bande d’onglets (panneau central — rythme Orders / Manual Orders).
pub fn workspace_tab_strip_trailer(ui: &mut Ui) {
    ui.add_space(4.0);
    ui.separator();
    ui.add_space(4.0);
}

/// Bande d’onglets dans la toolbar app (hauteur standard).
pub fn show_workspace_tab_strip_in_toolbar<'a>(
    ui: &mut Ui,
    tabs: &'a [WorkspaceTab<'a>],
    active_id: &str,
) -> Option<&'a str> {
    let style = workspace_tab_style_for_toolbar(ui);
    let mut picked = None;
    ui.allocate_ui_with_layout(
        egui::vec2(0.0, TOOLBAR_CONTROL_HEIGHT),
        egui::Layout::left_to_right(egui::Align::Center),
        |ui| {
            picked = show_workspace_tab_strip(ui, &style, tabs, active_id);
        },
    );
    picked
}