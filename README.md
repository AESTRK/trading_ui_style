# trading_ui_style


> **IPC** : registre [runtime/ipc.json](https://github.com/AESTRK/config_manager/blob/main/config/runtime/ipc.json) · inject `@ipc.<canal>` au boot (pas de port en dur dans le code). Voir [IPC_AND_LOGS.md](https://github.com/AESTRK/config_manager/blob/main/config/IPC_AND_LOGS.md).
> Crate Rust partagé — palettes, couleurs et formatters UI pour les applications de trading egui.

## Rôle dans l'écosystème

Bibliothèque de style commune consommée par les applications Rust (`orderbook`, `chartboard`, `statics_currency`, `portfolio`, `risk`, …). Centralise les couleurs achat/vente, les palettes clair/sombre, les bandeaux connectivité / issues et les helpers de formatage pour garantir une cohérence visuelle entre les outils.

## Entrées et sorties

### Fonctionnel

| Entrées | Sorties |
|---------|---------|
| Mode thème appelant (clair / sombre) | Palette cohérente buy/sell/mid/widgets |
| Valeurs numériques (pct, EUR, volume) | Chaînes formatées pour affichage UI |
| Variation % (positive / négative) | Couleur conditionnelle (`pct_rgb`) |

### Technique

| Entrées | Sorties |
|---------|---------|
| `ThemeMode` / `palette(mode)` | `TradingPalette`, `LIGHT_PALETTE`, `DARK_PALETTE` |
| `f64` (montants, volumes) | `String` via `format_pct`, `format_eur_compact`, `format_volume_compact` |
| — | `Rgb`, `TEXT_SIZES`, constantes `BUY_GREEN`, `SELL_RED`, `MID_BLUE` |

## Stack technique

| Composant | Détail |
|-----------|--------|
| Langage | Rust 2024 |
| Dépendances runtime | `egui` 0.31 (module `banner` uniquement) |
| Type | Bibliothèque (`lib` uniquement, pas de binaire) |

## API principale

| Export | Description |
|--------|-------------|
| `TradingPalette`, `LIGHT_PALETTE`, `DARK_PALETTE` | Couleurs thème (buy, sell, mid, widgets…) |
| `ThemeMode`, `palette()` | Sélection clair / sombre |
| `Rgb`, `BUY_GREEN`, `SELL_RED`, `MID_BLUE` | Couleurs de base |
| `TEXT_SIZES` | Tailles de police standardisées |
| `format_pct`, `format_eur_compact`, `format_volume_compact` | Formatters |
| `pct_rgb` | Couleur conditionnelle selon variation |
| `hide_dock_requested` | Lit `ALPHA_LAGOON_HIDE_DOCK` (masque l’icône Dock côté macOS) |
| `prepare_native_options` | Applique le masquage Dock sur `eframe::NativeOptions` |
| `run_native` | `eframe::run_native` + masquage Dock quand le launcher le demande |
| `apply_launcher_hide_dock!` | Compatibilité — préférer `run_native` |
| `banner::draw_feed_banner` | Bandeau réseau 2 lignes (titre, métriques, stats WS, bouton) |
| `banner::draw_secondary_banner` | Bandeau secondaire (carnets, API absente, connectivité) |
| `banner::WsDowntimeStats` | Dernière coupure / cumul / déconnexions |
| `app_issues` / `issues` | Bandeaux classifiés (connectivité, persist, …) avec `paint_top_stack_panel_bg` |
| `egui_theme::paint_top_stack_panel_bg` | Fond unifié derrière toolbar + bandeaux en haut de fenêtre |
| `egui_theme::top_stack_fill` | Couleur de fond du stack top (clair / sombre) |

## Launcher / Dock macOS

Le launcher Xcode définit `ALPHA_LAGOON_HIDE_DOCK=1` pour toute la stack (équivalent de `macos_dock.py` côté Python/Tk). Avec cette variable, les apps macOS passent `ActivationPolicy::Accessory` au `EventLoopBuilder` winit : **pas d’icône dans le Dock** pour chaque process (binaire non bundle).

Repli ponctuel si une version macOS / winit pose problème (fenêtre invisible) : définir **`ALPHA_LAGOON_GUI_REGULAR=1`** pour forcer une app normale avec icône Dock.

Chaque app GUI Rust doit lancer sa fenêtre via `trading_ui_style::run_native` — le masquage Dock suit `ALPHA_LAGOON_HIDE_DOCK` :

```rust
trading_ui_style::run_native(
    APP_TITLE,
    eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([width, height])
            .with_title(APP_TITLE),
        ..Default::default()
    },
    Box::new(|cc| {
        egui_theme::apply_system_visuals(&cc.egui_ctx);
        Ok(Box::new(MyApp::new()))
    }),
)?;
```

Aucune dépendance `winit` supplémentaire dans l'app : elle est portée par `trading_ui_style` (feature `launcher`, activée par défaut).

## Installation (consommateur)

Dépendance Git dans `Cargo.toml` :

```toml
trading_ui_style = { git = "https://github.com/AESTRK/trading_ui_style.git", branch = "main" }
```

## Build et tests

```bash
cargo build
cargo test
```

## Dépôt

https://github.com/AESTRK/trading_ui_style
