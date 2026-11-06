//! Politique d’activation macOS avant la boucle d’événements winit.
//!
//! Pour un binaire **non bundle** (pas de `CFBundleIdentifier`), winit force
//! `NSApplicationActivationPolicyRegular` dans `applicationDidFinishLaunching` si le delegate
//! n’a pas de politique explicite — d’où une icône Dock par process même avec notre hook
//! `EventLoopBuilder::with_activation_policy`. Voir winit `platform_impl/macos/app_state.rs`.

use objc2_app_kit::{NSApplication, NSApplicationActivationPolicy};
use objc2_foundation::MainThreadMarker;

/// Sans icône Dock ; doit être appelé sur le thread principal avant `EventLoop::build`.
pub(crate) fn apply_accessory_activation_policy() {
    let Some(mtm) = MainThreadMarker::new() else {
        tracing::warn!(
            "ALPHA_LAGOON_HIDE_DOCK: skip Dock hide — not on the main thread (NSApplication)"
        );
        return;
    };
    let app = NSApplication::sharedApplication(mtm);
    app.setActivationPolicy(NSApplicationActivationPolicy::Accessory);
}
