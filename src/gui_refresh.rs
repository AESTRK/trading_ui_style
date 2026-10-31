//! Planification repaint egui — policy suite ([`app_runtime_rust::gui_refresh`]).
use app_runtime_rust::{repaint_after_duration, resolve_hub_tick_ms, resolve_ui_refresh_ms};
pub use app_runtime_rust::GuiRefreshTier;
use eframe::egui;

/// Repaint planifié : persist `ui_refresh_ms` > 0 sinon suite (tier).
pub fn schedule_gui_refresh(
    ctx: &egui::Context,
    tier: GuiRefreshTier,
    persist_ui_refresh_ms: u64,
) {
    let ms = resolve_ui_refresh_ms(tier, persist_ui_refresh_ms);
    ctx.request_repaint_after(repaint_after_duration(ms));
}

pub fn schedule_gui_refresh_ms(ctx: &egui::Context, ms: u64) {
    ctx.request_repaint_after(repaint_after_duration(ms));
}

pub fn trading_tier_ms() -> u64 {
    GuiRefreshTier::Trading.suite_default_ms()
}

pub fn live_repaint_ms() -> u64 {
    GuiRefreshTier::Live.suite_default_ms()
}

pub fn orders_fast_poll_ms() -> u64 {
    trading_tier_ms()
}

pub fn supervision_repaint_ms(persist_ui_refresh_ms: u64) -> u64 {
    resolve_ui_refresh_ms(GuiRefreshTier::Supervision, persist_ui_refresh_ms)
}

pub fn portfolio_repaint_ms(ws_live: bool, ui_refresh_ms: u64, hub_tick_ms: u64) -> u64 {
    let ui = resolve_ui_refresh_ms(GuiRefreshTier::Desk, ui_refresh_ms);
    if ws_live {
        resolve_hub_tick_ms(hub_tick_ms, ui)
    } else {
        ui
    }
}

pub fn statics_instrument_map_repaint_ms() -> u64 {
    1000
}

pub fn manual_probe_step_ms() -> u64 {
    trading_tier_ms().saturating_add(20)
}

pub fn manual_probe_status_ms() -> u64 {
    manual_probe_step_ms().saturating_add(80)
}

const KLINES_INDEX_IDLE_MS: u64 = 750;
const KLINES_INDEX_ACTIVE_MS: u64 = 200;
const KLINES_IDLE_MS: u64 = 3000;

pub fn klines_repaint_ms(jobs_heavy: bool, index_idle_prix: bool, index_or_gap_catalogue: bool) -> u64 {
    if jobs_heavy {
        trading_tier_ms().saturating_add(20)
    } else if index_idle_prix {
        KLINES_INDEX_IDLE_MS
    } else if index_or_gap_catalogue {
        KLINES_INDEX_ACTIVE_MS
    } else {
        KLINES_IDLE_MS
    }
}

pub fn chartboard_repaint_ms(busy: bool, auto_refresh_sec: u64) -> u64 {
    if busy {
        trading_tier_ms()
    } else {
        let half = auto_refresh_sec.saturating_mul(1000) / 2;
        half.max(supervision_repaint_ms(0)).min(10_000)
    }
}
