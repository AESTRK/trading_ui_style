//! Menu toolbar **Export** — CSV/Excel (par app) + Export DB / Import DB (cache stack).

use std::sync::mpsc;
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

use app_runtime::export_db::{
    default_import_browse_dir, format_bytes_human, new_sink, run_cache_export_with_progress,
    run_cache_import, suggest_archive_path, target_for_app, CacheTarget, DbExportProgress,
    DbJobOutcome, ExportProgressSink,
};
use egui::{self, mutex::Mutex, Context};

use crate::toolbar::toolbar_menu_button;

#[derive(Clone)]
struct StackExportMenuHandle(Arc<Mutex<StackExportMenuPersist>>);

#[derive(Clone)]
struct ImportAckWatch {
    app_id: String,
    label: String,
    started: Instant,
}

#[derive(Default)]
struct StackExportMenuPersist {
    busy: bool,
    notice: Option<String>,
    job_rx: Option<mpsc::Receiver<DbJobOutcome>>,
    db_export_progress: Option<ExportProgressSink>,
    import_ack: Option<ImportAckWatch>,
    backup_before_import: bool,
    restart_after_import: bool,
}

impl StackExportMenuPersist {
    fn new() -> Self {
        Self {
            backup_before_import: true,
            restart_after_import: true,
            ..Default::default()
        }
    }
}

fn persist_id(app_id: &str) -> egui::Id {
    egui::Id::new(("stack_export_menu_v1", app_id))
}

fn handle(ctx: &Context, app_id: &str) -> StackExportMenuHandle {
    ctx.data_mut(|d| {
        d.get_temp_mut_or_insert_with(persist_id(app_id), || {
            StackExportMenuHandle(Arc::new(Mutex::new(StackExportMenuPersist::new())))
        })
        .clone()
    })
}

fn with_persist<R>(ctx: &Context, app_id: &str, f: impl FnOnce(&mut StackExportMenuPersist) -> R) -> R {
    let h = handle(ctx, app_id);
    f(&mut h.0.lock())
}

/// CSV / Excel déclenché depuis le menu Export (hors DB).
#[derive(Clone, Copy)]
pub struct SpreadsheetExport {
    pub enabled: bool,
    pub busy: bool,
}

/// Options export métier pour `show_app_toolbar_ex`.
pub struct ExportToolbarHook {
    pub spreadsheet: Option<SpreadsheetExport>,
}

impl ExportToolbarHook {
    pub fn spreadsheet_only(enabled: bool, busy: bool) -> Self {
        Self {
            spreadsheet: Some(SpreadsheetExport { enabled, busy }),
        }
    }
}

fn spreadsheet_trigger_id(app_id: &str) -> egui::Id {
    egui::Id::new(("spreadsheet_export_trigger", app_id))
}

fn set_spreadsheet_trigger(ctx: &Context, app_id: &str) {
    ctx.data_mut(|d| {
        *d.get_temp_mut_or_default(spreadsheet_trigger_id(app_id)) = true;
    });
}

/// Hook prêt à passer à [`crate::show_app_toolbar_ex`].
pub fn spreadsheet_export_hook(enabled: bool, busy: bool) -> ExportToolbarHook {
    ExportToolbarHook::spreadsheet_only(enabled, busy)
}

/// Après la toolbar : exécute `run` si l'utilisateur a choisi **Export CSV/Excel**.
pub fn if_spreadsheet_export_triggered(ctx: &Context, app_id: &str, run: impl FnOnce()) {
    if take_spreadsheet_export_trigger(ctx, app_id) {
        run();
    }
}

/// À appeler après `show_app_toolbar` si un export CSV/Excel a été demandé dans le menu.
pub fn take_spreadsheet_export_trigger(ctx: &Context, app_id: &str) -> bool {
    ctx.data_mut(|d| {
        let flag: &mut bool = d.get_temp_mut_or_default(spreadsheet_trigger_id(app_id));
        let hit = *flag;
        *flag = false;
        hit
    })
}

pub fn poll_stack_export_menu(ctx: &Context, app_id: &str) {
    with_persist(ctx, app_id, |state| {
        poll_job(state, app_id);
        poll_import_ack(state);
    });
    show_notice_window(ctx, app_id);
    if with_persist(ctx, app_id, |s| s.busy || s.import_ack.is_some()) {
        ctx.request_repaint_after(Duration::from_millis(200));
    }
}

pub fn should_show_export_menu(app_id: &str, hook: Option<&ExportToolbarHook>) -> bool {
    hook.and_then(|h| h.spreadsheet.as_ref())
        .is_some()
        || target_for_app(app_id).is_some()
}

pub fn draw_export_menu(
    ui: &mut egui::Ui,
    ctx: &Context,
    app_id: &str,
    hook: Option<&ExportToolbarHook>,
) -> bool {
    if !should_show_export_menu(app_id, hook) {
        return false;
    }
    let target = target_for_app(app_id);
    let busy = with_persist(ctx, app_id, |s| s.busy);
    let sheet_busy = hook.and_then(|h| h.spreadsheet).map(|s| s.busy).unwrap_or(false);
    let menu_label = if busy || sheet_busy {
        "Export…"
    } else {
        "Export"
    };
    let has_spreadsheet = hook.and_then(|h| h.spreadsheet).is_some();
    let sheet = hook.and_then(|h| h.spreadsheet);
    toolbar_menu_button(ui, menu_label, |ui| {
        if let Some(sheet) = sheet {
            let label = if sheet.busy {
                "Export CSV/Excel…"
            } else {
                "Export CSV/Excel"
            };
            if ui
                .add_enabled(sheet.enabled && !sheet.busy && !busy, egui::Button::new(label))
                .clicked()
            {
                set_spreadsheet_trigger(ctx, app_id);
                ui.close_menu();
            }
        }
        if let Some(t) = target.clone() {
            if has_spreadsheet {
                ui.separator();
            }
            if ui.add_enabled(!busy, egui::Button::new("Export DB")).clicked() {
                start_db_export(ctx, app_id, t.clone());
                ui.close_menu();
            }
            if ui.add_enabled(!busy, egui::Button::new("Import DB")).clicked() {
                start_db_import(ctx, app_id, t);
                ui.close_menu();
            }
        }
    });
    true
}

fn start_db_export(ctx: &Context, app_id: &str, target: CacheTarget) {
    let out = suggest_archive_path(&target);
    let root = app_runtime::export_db::cache_root_for(&target);
    let progress = new_sink();
    with_persist(ctx, app_id, |s| {
        s.db_export_progress = Some(progress.clone());
    });
    spawn_db_job(ctx, app_id, move || {
        run_cache_export_with_progress(target, root, out, Some(progress))
    });
}

fn start_db_import(ctx: &Context, app_id: &str, target: CacheTarget) {
    let browse = default_import_browse_dir(&target);
    let Some(archive) = rfd::FileDialog::new()
        .set_title("Import DB — archive .tar.gz")
        .set_directory(&browse)
        .add_filter("Archive", &["tar.gz", "tgz"])
        .pick_file()
    else {
        app_runtime::log_export_cancel(app_id, "db_import", "user_cancelled");
        with_persist(ctx, app_id, |s| {
            s.notice = Some("Import DB annulé.".into());
        });
        return;
    };
    let (backup, restart) = with_persist(ctx, app_id, |s| {
        (s.backup_before_import, s.restart_after_import)
    });
    let root = app_runtime::export_db::cache_root_for(&target);
    spawn_db_job(ctx, app_id, move || {
        run_cache_import(target, root, archive, backup, restart)
    });
}

fn spawn_db_job(ctx: &Context, app_id: &str, work: impl FnOnce() -> DbJobOutcome + Send + 'static) {
    let (tx, rx) = mpsc::channel();
    with_persist(ctx, app_id, |s| {
        s.busy = true;
        s.job_rx = Some(rx);
        s.notice = Some("Export / import DB en cours…".into());
    });
    thread::Builder::new()
        .name(format!("stack-export-{app_id}"))
        .spawn(move || {
            let outcome = work();
            let _ = tx.send(outcome);
        })
        .ok();
}

fn poll_job(state: &mut StackExportMenuPersist, menu_app_id: &str) {
    let Some(rx) = state.job_rx.as_ref() else {
        return;
    };
    if let Ok(outcome) = rx.try_recv() {
        state.busy = false;
        state.job_rx = None;
        state.db_export_progress = None;
        state.notice = Some(outcome.message);
        if let Some(ack) = outcome.import_ack {
            state.import_ack = Some(ImportAckWatch {
                app_id: ack.app_id,
                label: ack.label,
                started: Instant::now(),
            });
        }
        if let Some(path) = outcome.export_path {
            let deliver_app = outcome
                .deliver_app_id
                .as_deref()
                .unwrap_or(menu_app_id);
            let _ = app_runtime::deliver_export_outputs(deliver_app, &[&path]);
        }
    }
}

fn poll_import_ack(state: &mut StackExportMenuPersist) {
    let Some(watch) = state.import_ack.as_ref() else {
        return;
    };
    if watch.started.elapsed() > Duration::from_secs(180) {
        state.notice = Some(format!(
            "Import DB — pas encore de confirmation pour {} (voir CACHE_IMPORT_APPLIED).",
            watch.label
        ));
        state.import_ack = None;
        return;
    }
    if app_runtime::load_applied_ack(&watch.app_id).is_some() {
        state.notice = Some(format!(
            "Import DB confirmé au redémarrage de {} (CACHE_IMPORT_APPLIED).",
            watch.label
        ));
        state.import_ack = None;
    }
}

fn show_notice_window(ctx: &Context, app_id: &str) {
    let (text, waiting_ack, busy, progress) = with_persist(ctx, app_id, |s| {
        let progress = s
            .db_export_progress
            .as_ref()
            .and_then(|sink| sink.lock().ok().map(|g| g.clone()));
        (s.notice.clone(), s.import_ack.is_some(), s.busy, progress)
    });
    let Some(text) = text else {
        return;
    };
    let win_id = egui::Id::new(("export_notice", app_id));
    let mut open = true;
    egui::Window::new("Export")
        .id(win_id)
        .collapsible(false)
        .resizable(true)
        .default_width(480.0)
        .open(&mut open)
        .show(ctx, |ui| {
            if busy {
                if let Some(p) = progress {
                    paint_db_export_progress(ui, &p);
                    ui.separator();
                }
            }
            ui.label(&text);
            if waiting_ack {
                ui.horizontal(|ui| {
                    ui.spinner();
                    ui.label("En attente de confirmation au redémarrage…");
                });
            }
        });
    if !open {
        with_persist(ctx, app_id, |s| s.notice = None);
    }
}

fn paint_db_export_progress(ui: &mut egui::Ui, p: &DbExportProgress) {
    let phase_label = match p.phase.as_str() {
        "scan" => "Inventaire du cache",
        "archive" => "Archivage tar",
        "finish" => "Finalisation gzip",
        other => other,
    };
    ui.label(phase_label);
    match p.phase.as_str() {
        "scan" => {
            ui.label(format!(
                "{} fichiers · {}",
                p.files_done,
                format_bytes_human(p.bytes_done)
            ));
            ui.add(egui::ProgressBar::new(0.0).animate(true));
        }
        "archive" => {
            ui.label(format!(
                "{} / {} fichiers · {} / {}",
                p.files_done,
                p.files_total,
                format_bytes_human(p.bytes_done),
                format_bytes_human(p.bytes_total)
            ));
            let frac = if p.files_total > 0 {
                (p.files_done as f32 / p.files_total as f32).clamp(0.0, 1.0)
            } else {
                0.0
            };
            ui.add(egui::ProgressBar::new(frac).show_percentage());
        }
        "finish" => {
            ui.add(egui::ProgressBar::new(0.0).animate(true));
        }
        _ => {}
    }
    if !p.current_label.is_empty() {
        ui.small(
            egui::RichText::new(&p.current_label)
                .weak()
                .monospace(),
        );
    }
}