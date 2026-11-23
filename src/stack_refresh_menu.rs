//! Menu toolbar **Rafraîchir** — Recherche / profonde / UI / Close (+ extensions app).

use egui::Context;

use crate::toolbar::toolbar_menu_button;

/// Une entrée du menu (masquée si `visible` est faux).
#[derive(Clone, Copy)]
pub struct RefreshSlot {
    pub visible: bool,
    pub enabled: bool,
    pub busy: bool,
    pub tooltip: Option<&'static str>,
}

impl RefreshSlot {
    pub const HIDDEN: Self = Self {
        visible: false,
        enabled: false,
        busy: false,
        tooltip: None,
    };

    pub fn show(enabled: bool, busy: bool) -> Self {
        Self {
            visible: true,
            enabled,
            busy,
            tooltip: None,
        }
    }

    pub fn with_tooltip(mut self, text: &'static str) -> Self {
        self.tooltip = Some(text);
        self
    }
}

impl Default for RefreshSlot {
    fn default() -> Self {
        Self::HIDDEN
    }
}

/// Déclencheur métier (hors refresh standard).
pub type RefreshExtraMenuDraw = fn(&mut egui::Ui, &Context, &str);

/// Options pour [`crate::show_app_toolbar_ex`].
pub struct RefreshToolbarHook {
    pub recherche: RefreshSlot,
    pub recherche_profonde: RefreshSlot,
    pub rafraichissement_ui: RefreshSlot,
    pub rafraichir_close: RefreshSlot,
    /// Republication IPC globale (Risk — équivalent « Tous (possible) »).
    pub tous_les_flux_entrants: RefreshSlot,
    /// Symboles statics (klines_cache, …).
    pub recharger_statics: RefreshSlot,
    /// Index / métadonnées catalogue klines.
    pub maj_catalogue: RefreshSlot,
    /// Liste symboles statics / instrument map (close ref, manual orders, orderbook).
    pub recharger_referentiel: RefreshSlot,
    /// Relecture cache SQLite sans fetch venue (statics IPC, …).
    pub recharger_historique: RefreshSlot,
    /// Sous-menu app (ex. flux ZMQ ciblés Risk).
    pub extra_menu: Option<RefreshExtraMenuDraw>,
}

impl RefreshToolbarHook {
    pub fn cache_sync(idle: bool, with_close: bool, with_deep: bool) -> Self {
        Self {
            recherche: RefreshSlot::show(idle, !idle).with_tooltip(
                "Delta depuis le cache local — sync court API / watermark.",
            ),
            recherche_profonde: if with_deep {
                RefreshSlot::show(idle, !idle).with_tooltip(
                    "Resynchronisation complète sur la fenêtre configurée.",
                )
            } else {
                RefreshSlot::HIDDEN
            },
            rafraichissement_ui: RefreshSlot::show(true, false).with_tooltip(
                "Relecture locale et repaint — pas d'appel venue.",
            ),
            rafraichir_close: if with_close {
                RefreshSlot::show(idle, !idle).with_tooltip(
                    "Re-interroge close referential pour les lignes déjà en base.",
                )
            } else {
                RefreshSlot::HIDDEN
            },
            tous_les_flux_entrants: RefreshSlot::HIDDEN,
            recharger_statics: RefreshSlot::HIDDEN,
            maj_catalogue: RefreshSlot::HIDDEN,
            recharger_referentiel: RefreshSlot::HIDDEN,
            recharger_historique: RefreshSlot::HIDDEN,
            extra_menu: None,
        }
    }

    pub fn ui_only() -> Self {
        Self {
            recherche: RefreshSlot::HIDDEN,
            recherche_profonde: RefreshSlot::HIDDEN,
            rafraichissement_ui: RefreshSlot::show(true, false),
            rafraichir_close: RefreshSlot::HIDDEN,
            tous_les_flux_entrants: RefreshSlot::HIDDEN,
            recharger_statics: RefreshSlot::HIDDEN,
            maj_catalogue: RefreshSlot::HIDDEN,
            recharger_referentiel: RefreshSlot::HIDDEN,
            recharger_historique: RefreshSlot::HIDDEN,
            extra_menu: None,
        }
    }

    /// Delta / profonde historique ordres (REST venues).
    pub fn orders_history_sync(idle: bool) -> Self {
        Self {
            recherche: RefreshSlot::show(idle, !idle).with_tooltip(
                "Fenêtre courte — overlap cache · rattrapage frais.",
            ),
            recherche_profonde: RefreshSlot::show(idle, !idle).with_tooltip(
                "REST depuis history_deep_since_* (lourd).",
            ),
            rafraichissement_ui: RefreshSlot::show(true, false),
            ..RefreshToolbarHook::ui_only()
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RefreshAction {
    Recherche,
    RechercheProfonde,
    RafraichissementUi,
    RafraichirClose,
    TousLesFluxEntrants,
    RechargerStatics,
    MajCatalogue,
    RechargerReferentiel,
    RechargerHistorique,
}

fn trigger_id(app_id: &str, action: RefreshAction) -> egui::Id {
    let tag = match action {
        RefreshAction::Recherche => "recherche",
        RefreshAction::RechercheProfonde => "profonde",
        RefreshAction::RafraichissementUi => "ui",
        RefreshAction::RafraichirClose => "close",
        RefreshAction::TousLesFluxEntrants => "tous_flux",
        RefreshAction::RechargerStatics => "recharger_statics",
        RefreshAction::MajCatalogue => "maj_catalogue",
        RefreshAction::RechargerReferentiel => "recharger_ref",
        RefreshAction::RechargerHistorique => "recharger_hist",
    };
    egui::Id::new(("stack_refresh_trigger_v1", app_id, tag))
}

fn app_trigger_id(app_id: &str) -> egui::Id {
    egui::Id::new(("stack_refresh_app_trigger_v1", app_id))
}

fn set_trigger(ctx: &Context, app_id: &str, action: RefreshAction) {
    ctx.data_mut(|d| {
        *d.get_temp_mut_or_default(trigger_id(app_id, action)) = true;
    });
}

/// Code métier app (ex. variante flux Risk) — voir [`take_refresh_app_code`].
pub fn set_refresh_app_code(ctx: &Context, app_id: &str, code: u32) {
    ctx.data_mut(|d| {
        *d.get_temp_mut_or_default(app_trigger_id(app_id)) = Some(code);
    });
}

pub fn should_show_refresh_menu(hook: Option<&RefreshToolbarHook>) -> bool {
    hook.map(|h| {
        h.recherche.visible
            || h.recherche_profonde.visible
            || h.rafraichissement_ui.visible
            || h.rafraichir_close.visible
            || h.tous_les_flux_entrants.visible
            || h.recharger_statics.visible
            || h.maj_catalogue.visible
            || h.recharger_referentiel.visible
            || h.recharger_historique.visible
            || h.extra_menu.is_some()
    })
    .unwrap_or(false)
}

pub fn draw_refresh_menu(
    ui: &mut egui::Ui,
    ctx: &Context,
    app_id: &str,
    hook: Option<&RefreshToolbarHook>,
) -> bool {
    let Some(h) = hook else {
        return false;
    };
    if !should_show_refresh_menu(Some(h)) {
        return false;
    }
    let any_busy = h.recherche.busy
        || h.recherche_profonde.busy
        || h.rafraichir_close.busy
        || h.tous_les_flux_entrants.busy
        || h.recharger_statics.busy
        || h.maj_catalogue.busy
        || h.recharger_referentiel.busy
        || h.recharger_historique.busy;
    let menu_label = if any_busy { "Rafraîchir…" } else { "Rafraîchir" };
    toolbar_menu_button(ui, menu_label, |ui| {
        let draw_row = |ui: &mut egui::Ui, slot: RefreshSlot, action: RefreshAction, label: &'static str| {
            if !slot.visible {
                return;
            }
            let mut resp = ui.add_enabled_ui(slot.enabled && !slot.busy, |ui| {
                ui.selectable_label(false, label)
            }).inner;
            if let Some(tip) = slot.tooltip {
                resp = resp.on_hover_text(tip);
            }
            if resp.clicked() {
                set_trigger(ctx, app_id, action);
                ui.close_menu();
            }
        };
        draw_row(ui, h.recherche, RefreshAction::Recherche, "Recherche");
        draw_row(
            ui,
            h.recherche_profonde,
            RefreshAction::RechercheProfonde,
            "Recherche profonde",
        );
        draw_row(
            ui,
            h.rafraichissement_ui,
            RefreshAction::RafraichissementUi,
            "Rafraîchissement UI",
        );
        draw_row(
            ui,
            h.rafraichir_close,
            RefreshAction::RafraichirClose,
            "Rafraîchir Close",
        );
        let core_visible = h.recherche.visible
            || h.recherche_profonde.visible
            || h.rafraichissement_ui.visible
            || h.rafraichir_close.visible;
        let ext_visible = h.recharger_statics.visible
            || h.maj_catalogue.visible
            || h.recharger_referentiel.visible
            || h.recharger_historique.visible
            || h.tous_les_flux_entrants.visible
            || h.extra_menu.is_some();
        if ext_visible && core_visible {
            ui.separator();
        }
        draw_row(
            ui,
            h.recharger_statics,
            RefreshAction::RechargerStatics,
            "Recharger statics",
        );
        draw_row(ui, h.maj_catalogue, RefreshAction::MajCatalogue, "MAJ catalogue");
        draw_row(
            ui,
            h.recharger_referentiel,
            RefreshAction::RechargerReferentiel,
            "Recharger référentiel",
        );
        draw_row(
            ui,
            h.recharger_historique,
            RefreshAction::RechargerHistorique,
            "Recharger historique",
        );
        if (h.recharger_statics.visible
            || h.maj_catalogue.visible
            || h.recharger_referentiel.visible
            || h.recharger_historique.visible)
            && (h.tous_les_flux_entrants.visible || h.extra_menu.is_some())
        {
            ui.separator();
        }
        draw_row(
            ui,
            h.tous_les_flux_entrants,
            RefreshAction::TousLesFluxEntrants,
            "Tous les flux entrants",
        );
        if let Some(draw) = h.extra_menu {
            ui.menu_button("Flux ciblé", |ui| {
                draw(ui, ctx, app_id);
            });
        }
    });
    true
}

pub fn take_refresh_trigger(ctx: &Context, app_id: &str, action: RefreshAction) -> bool {
    ctx.data_mut(|d| {
        let flag: &mut bool = d.get_temp_mut_or_default(trigger_id(app_id, action));
        let hit = *flag;
        *flag = false;
        hit
    })
}

pub fn if_refresh_triggered(ctx: &Context, app_id: &str, action: RefreshAction, run: impl FnOnce()) {
    if take_refresh_trigger(ctx, app_id, action) {
        run();
    }
}

pub fn take_refresh_app_code(ctx: &Context, app_id: &str) -> Option<u32> {
    ctx.data_mut(|d| {
        let slot: &mut Option<u32> = d.get_temp_mut_or_default(app_trigger_id(app_id));
        slot.take()
    })
}