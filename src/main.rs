// Suppress dead_code warnings during initial development.
#![allow(dead_code)]

use std::sync::{Arc, Mutex};

mod config;
mod formatter;
mod harness;
mod keychain;
mod providers;
mod scheduler;
mod statusbar;
mod types;

use objc2_foundation::MainThreadMarker;
use objc2_app_kit::NSApplication;

use statusbar::{AppViewModel, StatusBarApp};

// Embed the icon at compile time — no runtime path resolution needed.
const ICON_BYTES: &[u8] = include_bytes!("../icons/statusbar_template.png");

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .target(env_logger::Target::Stderr)
        .init();

    log::info!("AI Coding Dashboard starting...");

    let mtm = MainThreadMarker::new().expect("must run on main thread");

    // ── Load config ──
    let cfg = config::load_config();
    let first_run = !cfg.first_run_completed;
    let selected_tools = if first_run {
        harness::scan_tools()
            .iter()
            .filter(|t| t.installed)
            .map(|t| t.id.clone())
            .collect()
    } else {
        cfg.selected_tools.clone()
    };

    let tools = harness::scan_tools();
    log::info!(
        "Scanned {} tools, {} installed",
        tools.len(),
        tools.iter().filter(|t| t.installed).count()
    );

    // ── Shared state ──
    let vm = Arc::new(Mutex::new(AppViewModel {
        first_run,
        tools,
        selected_tools,
        ..Default::default()
    }));

    // ── Build status bar ──
    let app = {
        let vm_guard = vm.lock().unwrap();
        StatusBarApp::new(mtm, ICON_BYTES, &vm_guard)
    };

    // Save config if first run
    if first_run {
        let mut new_cfg = cfg.clone();
        new_cfg.first_run_completed = true;
        new_cfg.selected_tools = {
            let vm_guard = vm.lock().unwrap();
            vm_guard.selected_tools.clone()
        };
        config::save_config(&new_cfg);
    }

    // Wire menu actions to the shared state
    StatusBarApp::wire_actions(&app, &vm, mtm);

    let app = Arc::new(app);

    // ── Background scheduler ──
    let app_clone = Arc::clone(&app);
    let vm_clone = Arc::clone(&vm);
    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().expect("tokio runtime");
        rt.block_on(scheduler::run(app_clone, vm_clone));
    });

    // Run event loop — NSApp::run never returns, terminates process on Quit
    log::info!("Entering Cocoa event loop");
    let ns_app = NSApplication::sharedApplication(mtm);

    // These Arcs are intentionally "leaked" — the app lives until the user quits
    // via the menu, at which point the process terminates.
    Box::leak(Box::new(app));
    drop(vm);

    ns_app.run();
}
