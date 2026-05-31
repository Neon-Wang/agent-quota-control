use crate::formatter;
use crate::types::{ServiceQuota, ToolInfo};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use objc2::rc::Retained;
use objc2::runtime::{AnyClass, AnyObject, ClassBuilder, Sel};
use objc2::{self, sel};
use objc2::{AnyThread, ClassType, MainThreadOnly};
use objc2_foundation::{MainThreadMarker, NSData, NSObject, NSString};
use objc2_app_kit::{
    NSApplication, NSApplicationActivationPolicy, NSImage, NSMenu, NSMenuItem, NSStatusBar,
};

// ── Shared state ──

#[derive(Default)]
pub struct AppViewModel {
    pub kimi_quota: Option<ServiceQuota>,
    pub codex_quota: Option<ServiceQuota>,
    pub tools: Vec<ToolInfo>,
    pub selected_tools: Vec<String>,
    pub first_run: bool,
}

impl AppViewModel {
    pub fn new() -> Self {
        Self::default()
    }
}

// ── ObjC delegate class ──

fn delegate_class() -> &'static AnyClass {
    static DELEGATE_CLASS: std::sync::OnceLock<&'static AnyClass> = std::sync::OnceLock::new();
    DELEGATE_CLASS.get_or_init(|| {
        let superclass = NSObject::class();
        let mut builder = ClassBuilder::new(c"KCSAppDelegate", superclass)
            .expect("Failed to allocate KCSAppDelegate class");

        extern "C" fn refresh_all(_this: &AnyObject, _cmd: Sel, _sender: *mut AnyObject) {
            show_alert_dispatch(
                "Refresh All",
                "Usage data is refreshed automatically every 5 minutes.",
            );
        }

        extern "C" fn manage_services(_this: &AnyObject, _cmd: Sel, _sender: *mut AnyObject) {
            show_alert_dispatch(
                "Manage Services",
                "Service configuration will be available in v0.2.\n\nCurrently monitoring:\n  \u{2022} Kimi Code\n  \u{2022} Codex (ChatGPT)",
            );
        }

        unsafe {
            builder.add_method(sel!(refreshAll:), refresh_all as extern "C" fn(_, _, _));
            builder.add_method(sel!(manageServices:), manage_services as extern "C" fn(_, _, _));
        }

        builder.register()
    })
}

fn make_delegate() -> Retained<AnyObject> {
    let cls = delegate_class();
    unsafe {
        let obj: *mut AnyObject = objc2::msg_send![cls, alloc];
        let obj: *mut AnyObject = objc2::msg_send![obj, init];
        Retained::from_raw(obj).expect("Failed to create delegate")
    }
}

// ── StatusBarApp ──

pub struct StatusBarApp {
    kimi_item: Retained<NSMenuItem>,
    codex_item: Retained<NSMenuItem>,
    updated_item: Retained<NSMenuItem>,
    _menu: Retained<NSMenu>,
    _status_item: Retained<objc2_app_kit::NSStatusItem>,
}

unsafe impl Send for StatusBarApp {}
unsafe impl Sync for StatusBarApp {}

impl StatusBarApp {
    pub fn new(mtm: MainThreadMarker, icon_bytes: &[u8], vm: &AppViewModel) -> Self {
        let app = NSApplication::sharedApplication(mtm);
        app.setActivationPolicy(NSApplicationActivationPolicy::Accessory);

        let statusbar = NSStatusBar::systemStatusBar();
        let status_item =
            statusbar.statusItemWithLength(objc2_app_kit::NSVariableStatusItemLength);

        // Set icon from embedded bytes
        if let Some(button) = status_item.button(mtm) {
            let nsdata = NSData::with_bytes(icon_bytes);
            let image = NSImage::initWithData(NSImage::alloc(), &nsdata);
            if let Some(image) = image {
                image.setTemplate(true);
                unsafe {
                    let _: () = objc2::msg_send![&*button, setImage: &*image];
                }
            } else {
                log::warn!("Failed to create NSImage from icon bytes");
            }
        }

        let (menu, kimi_item, codex_item, updated_item) =
            build_normal_menu(&status_item, mtm, vm);

        Self {
            kimi_item,
            codex_item,
            updated_item,
            _menu: menu,
            _status_item: status_item,
        }
    }

    pub fn wire_actions(app: &Self, _vm: &Arc<Mutex<AppViewModel>>, mtm: MainThreadMarker) {
        let ns_app = NSApplication::sharedApplication(mtm);

        unsafe {
            // Build delegate for custom actions
            let delegate = make_delegate();

            let menu_ptr: *const NSMenu = &*app._menu;
            let menu_ref: *mut AnyObject = menu_ptr as *mut _;
            let count: isize = objc2::msg_send![menu_ref, numberOfItems];
            let n = count as usize;

            let del_ptr: *mut AnyObject = &*delegate as *const _ as *mut _;
            let app_ptr: *mut AnyObject = &*ns_app as *const _ as *mut _;

            if n >= 3 {
                let item: *mut AnyObject = objc2::msg_send![menu_ref, itemAtIndex: (n - 3) as isize];
                let _: () = objc2::msg_send![item, setTarget: del_ptr];
                let _: () = objc2::msg_send![item, setAction: sel!(refreshAll:)];
            }
            if n >= 2 {
                let item: *mut AnyObject = objc2::msg_send![menu_ref, itemAtIndex: (n - 2) as isize];
                let _: () = objc2::msg_send![item, setTarget: del_ptr];
                let _: () = objc2::msg_send![item, setAction: sel!(manageServices:)];
            }
            {
                let item: *mut AnyObject = objc2::msg_send![menu_ref, itemAtIndex: (n - 1) as isize];
                let _: () = objc2::msg_send![item, setTarget: app_ptr];
                let _: () = objc2::msg_send![item, setAction: sel!(terminate:)];
            }

            // Leak delegate — it must stay alive for the app's lifetime
            let _ = Retained::into_raw(delegate);
        }

        log::info!("Menu actions wired");
    }

    pub fn schedule_update(app: &Arc<Self>, vm: &Arc<Mutex<AppViewModel>>) {
        let app = Arc::clone(app);
        let vm = Arc::clone(vm);
        dispatch::Queue::main().exec_async(move || {
            if let Ok(vm) = vm.lock() {
                app.update_labels(&vm);
            }
        });
    }

    fn update_labels(&self, vm: &AppViewModel) {
        self.kimi_item
            .setTitle(&NSString::from_str(&Self::format_quota_line("Kimi Code", &vm.kimi_quota)));
        self.codex_item
            .setTitle(&NSString::from_str(&Self::format_quota_line("Codex", &vm.codex_quota)));
        self.updated_item
            .setTitle(&NSString::from_str(&Self::format_last_updated(vm)));
    }

    fn format_quota_line(name: &str, quota: &Option<ServiceQuota>) -> String {
        match quota {
            Some(q) if q.success => {
                formatter::format_summary(&q.tiers)
                    .map(|s| format!("  {name}  {s}"))
                    .unwrap_or_else(|| format!("  {name}  No data"))
            }
            Some(q) if !q.credential_valid => {
                format!("  {name}  \u{26A0} {}", q.error.as_deref().unwrap_or("Not configured"))
            }
            Some(q) => {
                format!("  {name}  \u{26A0} {}", q.error.as_deref().unwrap_or("Query failed"))
            }
            None => format!("  {name}  \u{26AA} Loading..."),
        }
    }

    fn format_last_updated(vm: &AppViewModel) -> String {
        let now_ms = SystemTime::now()
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
            "  Last updated: never".into()
        } else {
            let secs_ago = ((now_ms - latest) as f64 / 1000.0) as u64;
            let time_str = if secs_ago < 60 {
                format!("{secs_ago}s ago")
            } else if secs_ago < 3600 {
                format!("{}m ago", secs_ago / 60)
            } else {
                format!("{}h ago", secs_ago / 3600)
            };
            format!("  Last updated: {time_str}")
        }
    }
}

fn show_alert_dispatch(title: &str, message: &str) {
    dispatch::Queue::main().exec_async({
        let title = title.to_string();
        let message = message.to_string();
        move || unsafe {
            // Get NSAlert class via objc runtime
            extern "C" {
                fn objc_getClass(name: *const std::ffi::c_char) -> *mut AnyObject;
            }
            let cls = objc_getClass(c"NSAlert".as_ptr() as *const std::ffi::c_char);
            let alert: *mut AnyObject = objc2::msg_send![cls, new];
            let t = NSString::from_str(&title);
            let m = NSString::from_str(&message);
            let _: () = objc2::msg_send![alert, setMessageText: &*t];
            let _: () = objc2::msg_send![alert, setInformativeText: &*m];
            let _: () = objc2::msg_send![alert, addButtonWithTitle: &*NSString::from_str("OK")];
            let _: isize = objc2::msg_send![alert, runModal];
        }
    });
}

// ── Menu builder ──

fn build_normal_menu(
    status_item: &objc2_app_kit::NSStatusItem,
    mtm: MainThreadMarker,
    vm: &AppViewModel,
) -> (
    Retained<NSMenu>,
    Retained<NSMenuItem>,
    Retained<NSMenuItem>,
    Retained<NSMenuItem>,
) {
    let menu = unsafe { build_menu(mtm, vm) };
    let menu_ref: &NSMenu = &menu;

    // Attach menu to status item now, before any retain shenanigans
    status_item.setMenu(Some(menu_ref));

    unsafe {
        let kimi_ptr: *mut AnyObject =
            objc2::msg_send![&menu as *const _ as *mut AnyObject, itemAtIndex: 1_isize];
        let codex_ptr: *mut AnyObject =
            objc2::msg_send![&menu as *const _ as *mut AnyObject, itemAtIndex: 2_isize];
        let updated_ptr: *mut AnyObject =
            objc2::msg_send![&menu as *const _ as *mut AnyObject, itemAtIndex: 3_isize];

        // Retain each item so it outlives the menu's autorelease pool
        let _: () = objc2::msg_send![kimi_ptr, retain];
        let _: () = objc2::msg_send![codex_ptr, retain];
        let _: () = objc2::msg_send![updated_ptr, retain];

        let kimi = Retained::from_raw(kimi_ptr as *mut NSMenuItem).expect("Kimi item");
        let codex = Retained::from_raw(codex_ptr as *mut NSMenuItem).expect("Codex item");
        let updated = Retained::from_raw(updated_ptr as *mut NSMenuItem).expect("Updated item");

        (menu, kimi, codex, updated)
    }
}

unsafe fn build_menu(mtm: MainThreadMarker, vm: &AppViewModel) -> Retained<NSMenu> {
    let menu = NSMenu::new(mtm);

    add_disabled(&menu, mtm, "Services");
    add_disabled(
        &menu,
        mtm,
        &StatusBarApp::format_quota_line("Kimi Code", &vm.kimi_quota),
    );
    add_disabled(
        &menu,
        mtm,
        &StatusBarApp::format_quota_line("Codex", &vm.codex_quota),
    );
    add_disabled(
        &menu,
        mtm,
        &StatusBarApp::format_last_updated(vm),
    );
    add_sep(&menu, mtm);

    add_disabled(&menu, mtm, "Harness Tools");
    for tool in &vm.tools {
        let icon = if tool.installed { "  \u{2713}" } else { "  \u{2717}" };
        add_disabled(&menu, mtm, &format!("{icon} {}", tool.name));
    }
    add_sep(&menu, mtm);

    add_action(&menu, mtm, "Refresh All");
    add_action(&menu, mtm, "Manage Services...");
    add_sep(&menu, mtm);
    add_action(&menu, mtm, "Quit");

    menu
}

unsafe fn add_disabled(menu: &NSMenu, mtm: MainThreadMarker, title: &str) {
    let item = NSMenuItem::initWithTitle_action_keyEquivalent(
        NSMenuItem::alloc(mtm),
        &NSString::from_str(title),
        None,
        &NSString::from_str(""),
    );
    item.setEnabled(false);
    menu.addItem(&item);
}

unsafe fn add_action(menu: &NSMenu, mtm: MainThreadMarker, title: &str) {
    let item = NSMenuItem::initWithTitle_action_keyEquivalent(
        NSMenuItem::alloc(mtm),
        &NSString::from_str(title),
        None,
        &NSString::from_str(""),
    );
    item.setEnabled(true);
    menu.addItem(&item);
}

fn add_sep(menu: &NSMenu, mtm: MainThreadMarker) {
    let sep = NSMenuItem::separatorItem(mtm);
    menu.addItem(&sep);
}
