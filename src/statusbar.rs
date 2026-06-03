use crate::types::{QuotaTier, ServiceQuota, ToolInfo};
use std::collections::HashSet;
use std::ffi::c_void;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use objc2::rc::Retained;
use objc2::runtime::{AnyClass, AnyObject, Sel};
use objc2::sel;
use objc2::MainThreadOnly;
use objc2_app_kit::{
    NSApplication, NSApplicationActivationPolicy, NSControlStateValueOff, NSControlStateValueOn,
    NSImage, NSMenu, NSMenuItem, NSStatusBar,
};
use objc2_foundation::{MainThreadMarker, NSString};

// ── Shared state ──

#[derive(Default)]
pub struct AppViewModel {
    pub kimi_quota: Option<ServiceQuota>,
    pub codex_quota: Option<ServiceQuota>,
    pub selected_services: Vec<String>,
    pub tools: Vec<ToolInfo>,
    pub selected_tools: Vec<String>,
    pub first_run: bool,
    pub last_refreshed_service: Option<String>,
}

pub struct StatusBarApp {
    status_item: Retained<objc2_app_kit::NSStatusItem>,
    _delegate: Retained<AnyObject>,
}

unsafe impl Send for StatusBarApp {}
unsafe impl Sync for StatusBarApp {}

struct AppContext {
    app: Arc<StatusBarApp>,
    vm: Arc<Mutex<AppViewModel>>,
}

static APP_CONTEXT: std::sync::OnceLock<AppContext> = std::sync::OnceLock::new();

pub fn set_app_context(app: Arc<StatusBarApp>, vm: Arc<Mutex<AppViewModel>>) {
    let _ = APP_CONTEXT.set(AppContext { app, vm });
}

// ── Icon loading ──

/// Load the .app icon via NSWorkspace iconForFile: and scale to 16x16 menu size.
fn load_app_icon(app_path: &str) -> Option<Retained<NSImage>> {
    unsafe {
        let ws_cls: *mut AnyObject = AnyClass::get(c"NSWorkspace").unwrap() as *const _ as *mut _;
        let ws: *mut AnyObject = objc2::msg_send![ws_cls, sharedWorkspace];
        if ws.is_null() {
            return None;
        }

        let ns_path = NSString::from_str(app_path);
        let icon_raw: *mut AnyObject = objc2::msg_send![ws, iconForFile: &*ns_path];
        if icon_raw.is_null() {
            return None;
        }

        // Scale to 16x16
        let size = objc2_foundation::NSSize {
            width: 16.0,
            height: 16.0,
        };
        let _: () = objc2::msg_send![icon_raw, setSize: size];

        // Retain and wrap — iconForFile returns autoreleased, Retained::retain bumps count
        Retained::retain(icon_raw as *mut NSImage)
    }
}

/// Try multiple common paths for an app icon.
fn find_app_icon(name: &str) -> Option<Retained<NSImage>> {
    let candidates = [
        format!("/Applications/{name}.app"),
        format!("/Applications/{name} CN.app"),
        format!("/System/Applications/{name}.app"),
    ];
    for p in &candidates {
        if std::path::Path::new(p).exists() {
            return load_app_icon(p);
        }
    }
    // Try broader: Kimi apps might be named differently
    if name == "Kimi" {
        for extra in &["Kimi For Coding", "KimiCode", "kimi"] {
            let p = format!("/Applications/{extra}.app");
            if std::path::Path::new(&p).exists() {
                return load_app_icon(&p);
            }
        }
    }
    if name == "Codex" {
        for extra in &["Codex CLI", "codex"] {
            let p = format!("/Applications/{extra}.app");
            if std::path::Path::new(&p).exists() {
                return load_app_icon(&p);
            }
        }
    }
    None
}

/// Cache key for an icon: uses the launch name or .app name.
fn get_or_load_icon(
    cache: &mut std::collections::HashMap<String, Retained<NSImage>>,
    name: &str,
) -> Option<*mut AnyObject> {
    if let Some(icon) = cache.get(name) {
        return Some(icon as *const _ as *mut _);
    }
    if let Some(icon) = find_app_icon(name) {
        let ptr = &*icon as *const NSImage as *mut AnyObject;
        cache.insert(name.to_string(), icon);
        return Some(ptr);
    }
    None
}

fn load_status_icon(vm: &AppViewModel) -> Option<Retained<NSImage>> {
    let kimi_icon = vm
        .selected_services
        .iter()
        .any(|service| service == "kimi")
        .then(|| find_app_icon("Kimi"))
        .flatten();
    let codex_icon = vm
        .selected_services
        .iter()
        .any(|service| service == "codex")
        .then(|| find_app_icon("Codex"))
        .flatten();

    match (kimi_icon, codex_icon) {
        (Some(kimi), Some(codex)) => compose_status_icons(&kimi, &codex).or(Some(kimi)),
        (Some(kimi), None) => Some(kimi),
        (None, Some(codex)) => Some(codex),
        (None, None) => load_fallback_status_icon(),
    }
}

#[allow(deprecated)]
fn compose_status_icons(left: &NSImage, right: &NSImage) -> Option<Retained<NSImage>> {
    unsafe {
        let size = objc2_foundation::NSSize {
            width: 34.0,
            height: 16.0,
        };
        let cls: *mut AnyObject = AnyClass::get(c"NSImage").unwrap() as *const _ as *mut _;
        let image_raw: *mut AnyObject = objc2::msg_send![cls, alloc];
        let image_raw: *mut AnyObject = objc2::msg_send![image_raw, initWithSize: size];
        if image_raw.is_null() {
            return None;
        }
        let image = Retained::from_raw(image_raw as *mut NSImage)?;

        image.lockFocus();
        let source = objc2_foundation::NSRect {
            origin: objc2_foundation::NSPoint { x: 0.0, y: 0.0 },
            size: objc2_foundation::NSSize {
                width: 0.0,
                height: 0.0,
            },
        };
        let left_rect = objc2_foundation::NSRect {
            origin: objc2_foundation::NSPoint { x: 0.0, y: 0.0 },
            size: objc2_foundation::NSSize {
                width: 16.0,
                height: 16.0,
            },
        };
        let right_rect = objc2_foundation::NSRect {
            origin: objc2_foundation::NSPoint { x: 18.0, y: 0.0 },
            size: objc2_foundation::NSSize {
                width: 16.0,
                height: 16.0,
            },
        };
        left.drawInRect_fromRect_operation_fraction(
            left_rect,
            source,
            objc2_app_kit::NSCompositingOperation::SourceOver,
            1.0,
        );
        right.drawInRect_fromRect_operation_fraction(
            right_rect,
            source,
            objc2_app_kit::NSCompositingOperation::SourceOver,
            1.0,
        );
        image.unlockFocus();

        Some(image)
    }
}

fn load_fallback_status_icon() -> Option<Retained<NSImage>> {
    let mut candidates = Vec::new();
    candidates.push(std::path::PathBuf::from("icons/statusbar_template.png"));
    if let Ok(exe) = std::env::current_exe() {
        if let Some(contents_dir) = exe.parent().and_then(|macos| macos.parent()) {
            candidates.push(contents_dir.join("Resources/statusbar_template.png"));
        }
    }

    for candidate in candidates {
        if candidate.exists() {
            if let Some(icon) = load_image_file(&candidate) {
                unsafe {
                    let size = objc2_foundation::NSSize {
                        width: 16.0,
                        height: 16.0,
                    };
                    let _: () =
                        objc2::msg_send![&*icon as *const NSImage as *mut AnyObject, setSize: size];
                    let _: () = objc2::msg_send![&*icon as *const NSImage as *mut AnyObject, setTemplate: true];
                }
                return Some(icon);
            }
        }
    }
    None
}

fn load_image_file(path: &std::path::Path) -> Option<Retained<NSImage>> {
    unsafe {
        let cls: *mut AnyObject = AnyClass::get(c"NSImage").unwrap() as *const _ as *mut _;
        let image_raw: *mut AnyObject = objc2::msg_send![cls, alloc];
        let ns_path = NSString::from_str(&path.to_string_lossy());
        let image_raw: *mut AnyObject =
            objc2::msg_send![image_raw, initWithContentsOfFile: &*ns_path];
        if image_raw.is_null() {
            return None;
        }
        Retained::from_raw(image_raw as *mut NSImage)
    }
}

// ── Raw ObjC delegate class ──

fn make_delegate() -> Retained<AnyObject> {
    static CLASS: std::sync::OnceLock<SendClass> = std::sync::OnceLock::new();
    let cls_ptr = CLASS
        .get_or_init(|| {
            let name = std::ffi::CString::new(format!("KCSDel{}", std::process::id())).unwrap();
            unsafe {
                let superclass: *mut AnyObject =
                    AnyClass::get(c"NSObject").unwrap() as *const _ as *mut _;
                let cls = objc_allocateClassPair(superclass, name.as_ptr(), 0);
                if cls.is_null() {
                    panic!("objc_allocateClassPair failed");
                }
                let s = sel!(openTool:);
                class_addMethod(cls, s, open_tool_impl as *mut c_void, c"v@:@".as_ptr());
                let s = sel!(openCli:);
                class_addMethod(cls, s, open_cli_impl as *mut c_void, c"v@:@".as_ptr());
                let s = sel!(toggleTool:);
                class_addMethod(cls, s, toggle_tool_impl as *mut c_void, c"v@:@".as_ptr());
                let s = sel!(refreshNow:);
                class_addMethod(cls, s, refresh_now_impl as *mut c_void, c"v@:@".as_ptr());
                let s = sel!(setKimiKey:);
                class_addMethod(cls, s, set_key_impl as *mut c_void, c"v@:@".as_ptr());
                objc_registerClassPair(cls);
                SendClass(cls)
            }
        })
        .0;

    unsafe {
        let obj: *mut AnyObject = objc2::msg_send![cls_ptr, alloc];
        let obj: *mut AnyObject = objc2::msg_send![obj, init];
        Retained::from_raw(obj).expect("delegate")
    }
}

extern "C" {
    fn objc_allocateClassPair(
        superclass: *mut AnyObject,
        name: *const std::os::raw::c_char,
        extra_bytes: usize,
    ) -> *mut AnyObject;
    fn objc_registerClassPair(cls: *mut AnyObject);
    fn class_addMethod(
        cls: *mut AnyObject,
        name: Sel,
        imp: *mut c_void,
        types: *const std::os::raw::c_char,
    ) -> bool;
}

struct SendClass(*mut AnyObject);
unsafe impl Send for SendClass {}
unsafe impl Sync for SendClass {}

// ── Delegate impls ──

extern "C" fn toggle_tool_impl(_this: &AnyObject, _cmd: Sel, sender: *mut AnyObject) {
    if sender.is_null() {
        return;
    }
    let obj: *mut AnyObject = unsafe { objc2::msg_send![sender, representedObject] };
    if obj.is_null() {
        return;
    }
    let tool_id = unsafe { (*(obj as *const NSString)).to_string() };

    let Some(ctx) = APP_CONTEXT.get() else {
        return;
    };
    {
        let mut vm = match ctx.vm.lock() {
            Ok(vm) => vm,
            Err(e) => {
                log::error!("Failed to lock view model while toggling tool: {e}");
                return;
            }
        };

        if let Some(pos) = vm.selected_tools.iter().position(|id| id == &tool_id) {
            vm.selected_tools.remove(pos);
        } else {
            vm.selected_tools.push(tool_id);
            vm.selected_tools.sort();
            vm.selected_tools.dedup();
        }

        let mut config = crate::config::load_config();
        config.selected_tools = vm.selected_tools.clone();
        crate::config::save_config(&config);
    }

    StatusBarApp::schedule_update(&ctx.app, &ctx.vm);
}

extern "C" fn open_tool_impl(_this: &AnyObject, _cmd: Sel, sender: *mut AnyObject) {
    if sender.is_null() {
        return;
    }
    let obj: *mut AnyObject = unsafe { objc2::msg_send![sender, representedObject] };
    if obj.is_null() {
        return;
    }
    let name = unsafe { (*(obj as *const NSString)).to_string() };
    std::thread::spawn(move || {
        // open -a without -n: activates existing instance, no Dock duplicates
        let _ = std::process::Command::new("open")
            .arg("-a")
            .arg(&name)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn();
    });
}

/// CLI tool: pick folder → Ghostty new window (reuses existing instance) or Terminal fallback.
extern "C" fn open_cli_impl(_this: &AnyObject, _cmd: Sel, sender: *mut AnyObject) {
    if sender.is_null() {
        return;
    }
    let obj: *mut AnyObject = unsafe { objc2::msg_send![sender, representedObject] };
    if obj.is_null() {
        return;
    }
    let cmd = unsafe { (*(obj as *const NSString)).to_string() };

    if let Some(dir) = pick_folder() {
        let shell_cmd = format!("cd {} && {}", shell_quote(&dir), shell_quote(&cmd));

        let ghostty_app = "/Applications/Ghostty.app";
        if std::path::Path::new(ghostty_app).exists() {
            // open -a Ghostty --args -e ... → reuses existing Ghostty, opens new window
            let _ = std::process::Command::new("open")
                .arg("-a")
                .arg("Ghostty")
                .arg("--args")
                .arg("-e")
                .arg("/bin/zsh")
                .arg("-c")
                .arg(&shell_cmd)
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .spawn();
            return;
        }

        // Fallback: Terminal.app via AppleScript
        let script = format!(
            "tell application \"Terminal\" to do script {}",
            applescript_string(&shell_cmd)
        );
        let _ = std::process::Command::new("osascript")
            .arg("-e")
            .arg(&script)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn();
    }
}

/// Show NSOpenPanel to pick a directory. Returns the selected path.
fn pick_folder() -> Option<String> {
    unsafe {
        let cls: *mut AnyObject = AnyClass::get(c"NSOpenPanel").unwrap() as *const _ as *mut _;
        let panel: *mut AnyObject = objc2::msg_send![cls, openPanel];
        let _: () = objc2::msg_send![panel, setCanChooseFiles: false];
        let _: () = objc2::msg_send![panel, setCanChooseDirectories: true];
        let _: () = objc2::msg_send![panel, setCanCreateDirectories: false];
        let _: () = objc2::msg_send![panel, setAllowsMultipleSelection: false];
        let _: () =
            objc2::msg_send![panel, setTitle: &*NSString::from_str("Choose project folder")];
        let _: () = objc2::msg_send![panel, setMessage: &*NSString::from_str("Select a folder to open in terminal with this CLI tool")];
        let response: isize = objc2::msg_send![panel, runModal];
        if response == 1 {
            // NSModalResponseOK = 1
            let urls: *mut AnyObject = objc2::msg_send![panel, URLs];
            let count: usize = objc2::msg_send![urls, count];
            if count > 0 {
                let url: *mut AnyObject = objc2::msg_send![urls, objectAtIndex: 0];
                if !url.is_null() {
                    let path: *mut AnyObject = objc2::msg_send![url, path];
                    if !path.is_null() {
                        return Some((*(path as *const NSString)).to_string());
                    }
                }
            }
        }
    }
    None
}

fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}

fn applescript_string(value: &str) -> String {
    format!("\"{}\"", value.replace('\\', "\\\\").replace('"', "\\\""))
}

extern "C" fn refresh_now_impl(_this: &AnyObject, _cmd: Sel, _sender: *mut AnyObject) {
    show_alert_str(
        "Refresh",
        "Auto-refresh runs every 5 minutes. Data updates automatically.",
    );
}

extern "C" fn set_key_impl(_this: &AnyObject, _cmd: Sel, _sender: *mut AnyObject) {
    // Menu action callbacks run on main thread — show alert directly
    let key = show_key_input_alert();
    if let Some(key) = key {
        let key = key.trim().to_string();
        if !key.is_empty() {
            if let Err(e) = crate::keychain::store_api_key(
                crate::keychain::KIMI_SERVICE,
                crate::keychain::KIMI_ACCOUNT,
                &key,
            ) {
                show_alert_str("Error", &format!("Failed to save: {e}"));
            } else {
                show_alert_str("Saved", "Kimi API key saved.\nRestart to see usage data.");
            }
        }
    }
}

fn show_key_input_alert() -> Option<String> {
    unsafe {
        let cls: *mut AnyObject = AnyClass::get(c"NSAlert").unwrap() as *const _ as *mut _;
        let alert: *mut AnyObject = objc2::msg_send![cls, new];
        let _: () =
            objc2::msg_send![alert, setMessageText: &*NSString::from_str("Set Kimi Code API Key")];
        let _: () = objc2::msg_send![alert, setInformativeText: &*NSString::from_str("Enter your Kimi API key (starts with sk-)")];

        // Create input text field
        let tf_cls: *mut AnyObject = AnyClass::get(c"NSTextField").unwrap() as *const _ as *mut _;
        let rect = objc2_foundation::NSRect {
            origin: objc2_foundation::NSPoint { x: 0.0, y: 0.0 },
            size: objc2_foundation::NSSize {
                width: 300.0,
                height: 24.0,
            },
        };
        let tf: *mut AnyObject = objc2::msg_send![tf_cls, alloc];
        let tf: *mut AnyObject = objc2::msg_send![tf, initWithFrame: rect];
        let _: () = objc2::msg_send![tf, setStringValue: &*NSString::from_str("")];
        let _: () =
            objc2::msg_send![tf, setPlaceholderString: &*NSString::from_str("sk-xxxxxxxxxxxxxxxx")];
        let _: () = objc2::msg_send![alert, setAccessoryView: tf];

        let _: () = objc2::msg_send![alert, addButtonWithTitle: &*NSString::from_str("Save")];
        let _: () = objc2::msg_send![alert, addButtonWithTitle: &*NSString::from_str("Cancel")];

        let response: isize = objc2::msg_send![alert, runModal];
        if response == 1000 {
            // NSAlertFirstButtonReturn
            let value: *mut AnyObject = objc2::msg_send![tf, stringValue];
            if !value.is_null() {
                let s = (*(value as *const NSString)).to_string();
                return Some(s);
            }
        }
    }
    None
}

fn show_alert_str(title: &str, msg: &str) {
    unsafe {
        let cls: *mut AnyObject = AnyClass::get(c"NSAlert").unwrap() as *const _ as *mut _;
        let a: *mut AnyObject = objc2::msg_send![cls, new];
        let _: () = objc2::msg_send![a, setMessageText: &*NSString::from_str(title)];
        let _: () = objc2::msg_send![a, setInformativeText: &*NSString::from_str(msg)];
        let _: () = objc2::msg_send![a, addButtonWithTitle: &*NSString::from_str("OK")];
        let _: () = objc2::msg_send![a, runModal];
    }
}

// ── StatusBarApp ──

impl StatusBarApp {
    pub fn new(mtm: MainThreadMarker, vm: &AppViewModel) -> Self {
        NSApplication::sharedApplication(mtm)
            .setActivationPolicy(NSApplicationActivationPolicy::Accessory);

        let delegate = make_delegate();
        let del_ptr: *mut AnyObject = &*delegate as *const _ as *mut _;

        let statusbar = NSStatusBar::systemStatusBar();
        let status_item = statusbar.statusItemWithLength(objc2_app_kit::NSVariableStatusItemLength);

        // Set icon + text in menu bar (icon left, text right)
        if let Some(button) = status_item.button(mtm) {
            unsafe {
                let text = Self::bar_text(vm);
                let _: () = objc2::msg_send![&*button, setTitle: &*NSString::from_str(&text)];

                // Load Kimi and Codex icons, compacted into one menu bar image when both exist.
                if let Some(ref icon) = load_status_icon(vm) {
                    let _: () = objc2::msg_send![&*button, setImage: &**icon as *const NSImage as *mut AnyObject];
                    // NSImageLeft = 2 (NSButton imagePosition enum)
                    let _: () = objc2::msg_send![&*button, setImagePosition: 2_usize];
                }

                let _: () = objc2::msg_send![&*button, setToolTip: &*NSString::from_str("AI Coding Dashboard")];
            }
        }

        let menu = Self::build_menu(mtm, vm, del_ptr);
        status_item.setMenu(Some(&menu));

        Self {
            status_item,
            _delegate: delegate,
        }
    }

    pub fn schedule_update(app: &Arc<Self>, vm: &Arc<Mutex<AppViewModel>>) {
        let app = Arc::clone(app);
        let vm = Arc::clone(vm);
        dispatch::Queue::main().exec_async(move || {
            if let Ok(guard) = vm.lock() {
                app.update(&guard);
            }
        });
    }

    fn update(&self, vm: &AppViewModel) {
        let mtm = MainThreadMarker::new().expect("main thread");
        if let Some(button) = self.status_item.button(mtm) {
            unsafe {
                let text = Self::bar_text(vm);
                let _: () = objc2::msg_send![&*button, setTitle: &*NSString::from_str(&text)];

                if let Some(ref icon) = load_status_icon(vm) {
                    let _: () = objc2::msg_send![&*button, setImage: &**icon as *const NSImage as *mut AnyObject];
                    let _: () = objc2::msg_send![&*button, setImagePosition: 2_usize];
                }
            }
        }
        let del_ptr: *mut AnyObject = &*self._delegate as *const _ as *mut _;
        let menu = Self::build_menu(mtm, vm, del_ptr);
        self.status_item.setMenu(Some(&menu));
    }

    /// Menu bar text: shows the tightest 5-hour percentage plus overall sufficiency.
    fn bar_text(vm: &AppViewModel) -> String {
        let now = crate::estimator::now_unix_secs();
        let state =
            crate::estimator::overall_state_for_services([&vm.kimi_quota, &vm.codex_quota], now);
        let five_hour_pct = [&vm.kimi_quota, &vm.codex_quota]
            .into_iter()
            .filter_map(|q| q.as_ref())
            .filter(|q| q.success)
            .flat_map(|q| q.tiers.iter())
            .filter(|t| t.name == "five_hour")
            .map(|t| t.utilization)
            .reduce(f64::max);

        match five_hour_pct {
            Some(pct) => format!("h{:.0}% {}", pct.round(), state.label()),
            None => format!("h--% {}", state.label()),
        }
    }

    fn build_menu(
        mtm: MainThreadMarker,
        vm: &AppViewModel,
        del_ptr: *mut AnyObject,
    ) -> Retained<NSMenu> {
        let menu = NSMenu::new(mtm);

        // ── Status at a glance ──
        let has_kimi = vm.kimi_quota.as_ref().is_some_and(|q| q.credential_valid);
        let has_codex = vm.codex_quota.as_ref().is_some_and(|q| q.credential_valid);
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as i64;

        let kimi_age = vm
            .kimi_quota
            .as_ref()
            .and_then(|q| q.queried_at)
            .map(|t| now - t)
            .unwrap_or(i64::MAX);
        let codex_age = vm
            .codex_quota
            .as_ref()
            .and_then(|q| q.queried_at)
            .map(|t| now - t)
            .unwrap_or(i64::MAX);
        let monitoring = if kimi_age < codex_age {
            "Kimi Code"
        } else {
            "Codex"
        };
        Self::dlbl(&menu, mtm, &format!("Monitoring: {monitoring}"));
        Self::sep(&menu, mtm);

        // ── Usage Detail ──
        if has_kimi {
            Self::service_section(&menu, mtm, "Kimi Code", &vm.kimi_quota);
        }
        if has_codex {
            Self::service_section(&menu, mtm, "Codex", &vm.codex_quota);
        }
        if !has_kimi && !has_codex {
            Self::dlbl(&menu, mtm, "  Configure API keys to see usage");
        }
        Self::dlbl(&menu, mtm, &Self::fmt_updated(vm));
        Self::sep(&menu, mtm);

        // ── API Keys ──
        if !has_kimi {
            Self::act_with_target(
                &menu,
                mtm,
                "\u{26A0} Set Kimi API Key...",
                sel!(setKimiKey:),
                del_ptr,
            );
        }
        if !has_codex {
            Self::dlbl(
                &menu,
                mtm,
                "\u{26A0} Codex: login with 'codex login' in terminal",
            );
        }
        Self::sep(&menu, mtm);

        // ── Tool Selection ──
        Self::dlbl(&menu, mtm, "Tool Selection");
        if vm.tools.is_empty() {
            Self::dlbl(&menu, mtm, "  No detected tools");
        } else {
            let selected = Self::selected_tool_set(vm);
            for tool in &vm.tools {
                Self::tool_selection_item(
                    &menu,
                    mtm,
                    tool,
                    selected.contains(tool.id.as_str()),
                    sel!(toggleTool:),
                    del_ptr,
                );
            }
        }
        Self::sep(&menu, mtm);

        let selected = Self::selected_tool_set(vm);
        if selected.is_empty() {
            Self::dlbl(&menu, mtm, "No tools selected");
            Self::sep(&menu, mtm);
        }

        // ── IDEs ──
        let ides: Vec<_> = vm
            .tools
            .iter()
            .filter(|t| t.tool_type == crate::types::ToolType::IDE)
            .filter(|t| selected.contains(t.id.as_str()))
            .collect();
        if !ides.is_empty() {
            Self::dlbl(&menu, mtm, "IDE & Apps");
            for tool in &ides {
                let launch_name = tool.launch_as.as_deref().unwrap_or(&tool.name);
                Self::tool_item(
                    &menu,
                    mtm,
                    launch_name,
                    launch_name,
                    sel!(openTool:),
                    del_ptr,
                );
            }
            Self::sep(&menu, mtm);
        }

        // ── CLI Tools ──
        let clis: Vec<_> = vm
            .tools
            .iter()
            .filter(|t| t.tool_type == crate::types::ToolType::CLI)
            .filter(|t| selected.contains(t.id.as_str()))
            .collect();
        if !clis.is_empty() {
            Self::dlbl(&menu, mtm, "CLI Tools (pick folder → Ghostty)");
            for tool in &clis {
                let bin = tool.install_path.as_deref().unwrap_or(&tool.name);
                Self::cli_tool_item(&menu, mtm, &tool.name, bin, sel!(openCli:), del_ptr);
            }
        }
        Self::sep(&menu, mtm);

        // ── Actions ──
        Self::act_with_target(&menu, mtm, "Refresh Now", sel!(refreshNow:), del_ptr);
        Self::sep(&menu, mtm);
        Self::act(&menu, mtm, "Quit", Some(sel!(terminate:)));

        menu
    }

    fn service_section(menu: &NSMenu, mtm: MainThreadMarker, name: &str, q: &Option<ServiceQuota>) {
        if let Some(q) = q {
            if q.success && !q.tiers.is_empty() {
                let now = crate::estimator::now_unix_secs();
                for t in &q.tiers {
                    let label = match t.name.as_str() {
                        "five_hour" => "5-Hour",
                        "weekly_limit" | "seven_day" => "7-Day",
                        _ => &t.name,
                    };
                    let pct = format!("{:.0}%", t.utilization.round());
                    let cd = Self::tier_countdown(t);
                    let line = if cd.is_empty() {
                        format!("    {label}: {pct}")
                    } else {
                        format!("    {label}: {pct}  (resets {cd})")
                    };
                    Self::dlbl(menu, mtm, &line);
                    Self::dlbl(menu, mtm, &Self::estimate_line(t, now));
                }
            } else {
                Self::dlbl(
                    menu,
                    mtm,
                    &format!(
                        "  {}  {}",
                        name,
                        q.error.as_deref().unwrap_or("Query failed")
                    ),
                );
            }
        } else {
            Self::dlbl(menu, mtm, &format!("  {}  Loading...", name));
        }
    }

    fn estimate_line(tier: &QuotaTier, now: i64) -> String {
        let estimate = crate::estimator::estimate_tier(tier, now);
        match estimate.projected_utilization {
            Some(projected) => format!(
                "      Estimate: {} · projected {:.0}%",
                estimate.state.label(),
                projected
            ),
            None => format!("      Estimate: {}", estimate.state.label()),
        }
    }

    fn selected_tool_set(vm: &AppViewModel) -> HashSet<&str> {
        vm.selected_tools.iter().map(String::as_str).collect()
    }

    fn tool_selection_item(
        menu: &NSMenu,
        mtm: MainThreadMarker,
        tool: &ToolInfo,
        checked: bool,
        action: Sel,
        target: *mut AnyObject,
    ) {
        let kind = match tool.tool_type {
            crate::types::ToolType::CLI => "CLI",
            crate::types::ToolType::IDE => "IDE",
        };
        let label = format!("    {kind}: {}", tool.name);
        let item = unsafe {
            NSMenuItem::initWithTitle_action_keyEquivalent(
                NSMenuItem::alloc(mtm),
                &NSString::from_str(&label),
                Some(action),
                &NSString::from_str(""),
            )
        };
        item.setEnabled(true);
        item.setState(if checked {
            NSControlStateValueOn
        } else {
            NSControlStateValueOff
        });
        unsafe {
            let _: () =
                objc2::msg_send![&*item as *const NSMenuItem as *mut AnyObject, setTarget: target];
            let ns = NSString::from_str(&tool.id);
            let _: () = objc2::msg_send![&*item as *const NSMenuItem as *mut AnyObject, setRepresentedObject: &*ns];
        }
        menu.addItem(&item);
    }

    /// CLI tool menu item — click picks folder then opens in Ghostty.
    fn cli_tool_item(
        menu: &NSMenu,
        mtm: MainThreadMarker,
        name: &str,
        cmd: &str,
        action: Sel,
        target: *mut AnyObject,
    ) {
        let label = format!("    {}", name);
        let item = unsafe {
            NSMenuItem::initWithTitle_action_keyEquivalent(
                NSMenuItem::alloc(mtm),
                &NSString::from_str(&label),
                Some(action),
                &NSString::from_str(""),
            )
        };
        item.setEnabled(true);
        unsafe {
            let _: () =
                objc2::msg_send![&*item as *const NSMenuItem as *mut AnyObject, setTarget: target];
            // representedObject stores the CLI command to run
            let ns = NSString::from_str(cmd);
            let _: () = objc2::msg_send![&*item as *const NSMenuItem as *mut AnyObject, setRepresentedObject: &*ns];
        }
        menu.addItem(&item);
    }

    /// Menu item with the tool's .app icon loaded from /Applications.
    fn tool_item(
        menu: &NSMenu,
        mtm: MainThreadMarker,
        _display_name: &str,
        launch_name: &str,
        action: Sel,
        target: *mut AnyObject,
    ) {
        // Try loading icon from /Applications/<launch_name>.app
        let app_path = format!("/Applications/{launch_name}.app");
        let icon = load_app_icon(&app_path);

        let label = format!("    {}", launch_name);
        let item = unsafe {
            NSMenuItem::initWithTitle_action_keyEquivalent(
                NSMenuItem::alloc(mtm),
                &NSString::from_str(&label),
                Some(action),
                &NSString::from_str(""),
            )
        };
        item.setEnabled(true);

        if let Some(ref icon) = icon {
            item.setImage(Some(&**icon));
        }

        unsafe {
            let _: () =
                objc2::msg_send![&*item as *const NSMenuItem as *mut AnyObject, setTarget: target];
            let ns = NSString::from_str(launch_name);
            let _: () = objc2::msg_send![&*item as *const NSMenuItem as *mut AnyObject, setRepresentedObject: &*ns];
        }
        menu.addItem(&item);
    }

    fn tier_countdown(t: &QuotaTier) -> String {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;
        if let Some(ref r) = t.resets_at {
            if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(r) {
                let rem = dt.timestamp() - now;
                if rem <= 0 {
                    return String::new();
                }
                if rem < 3600 {
                    format!("{}m", rem / 60)
                } else if rem < 86400 {
                    format!("{}h{}m", rem / 3600, (rem % 3600) / 60)
                } else {
                    format!("{}d{}h", rem / 86400, (rem % 86400) / 3600)
                }
            } else {
                String::new()
            }
        } else {
            String::new()
        }
    }

    fn fmt_updated(vm: &AppViewModel) -> String {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as i64;
        let latest = [&vm.kimi_quota, &vm.codex_quota]
            .iter()
            .filter_map(|q| q.as_ref())
            .filter_map(|q| q.queried_at)
            .max()
            .unwrap_or(0);
        if latest == 0 {
            "  Updated: never".into()
        } else {
            let s = ((now - latest) as f64 / 1000.0) as u64;
            let t = if s < 60 {
                format!("{s}s ago")
            } else if s < 3600 {
                format!("{}m ago", s / 60)
            } else {
                format!("{}h ago", s / 3600)
            };
            format!("  Updated: {t}")
        }
    }

    // ── Helpers ──

    fn dlbl(menu: &NSMenu, mtm: MainThreadMarker, title: &str) {
        let item = unsafe {
            NSMenuItem::initWithTitle_action_keyEquivalent(
                NSMenuItem::alloc(mtm),
                &NSString::from_str(title),
                None,
                &NSString::from_str(""),
            )
        };
        item.setEnabled(false);
        menu.addItem(&item);
    }

    fn act(menu: &NSMenu, mtm: MainThreadMarker, title: &str, action: Option<Sel>) {
        let item = unsafe {
            NSMenuItem::initWithTitle_action_keyEquivalent(
                NSMenuItem::alloc(mtm),
                &NSString::from_str(title),
                action,
                &NSString::from_str(""),
            )
        };
        item.setEnabled(true);
        menu.addItem(&item);
    }

    fn act_with_target(
        menu: &NSMenu,
        mtm: MainThreadMarker,
        title: &str,
        action: Sel,
        target: *mut AnyObject,
    ) {
        let item = unsafe {
            NSMenuItem::initWithTitle_action_keyEquivalent(
                NSMenuItem::alloc(mtm),
                &NSString::from_str(title),
                Some(action),
                &NSString::from_str(""),
            )
        };
        item.setEnabled(true);
        unsafe {
            let _: () =
                objc2::msg_send![&*item as *const NSMenuItem as *mut AnyObject, setTarget: target];
        }
        menu.addItem(&item);
    }

    fn sep(menu: &NSMenu, mtm: MainThreadMarker) {
        menu.addItem(&NSMenuItem::separatorItem(mtm));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shell_quote_handles_spaces_and_single_quotes() {
        assert_eq!(
            shell_quote("/Users/test/Project's Folder"),
            "'/Users/test/Project'\\''s Folder'"
        );
    }

    #[test]
    fn applescript_string_escapes_quotes_and_backslashes() {
        assert_eq!(
            applescript_string("cd \"/Users/test\" && echo \\ok"),
            "\"cd \\\"/Users/test\\\" && echo \\\\ok\""
        );
    }
}
