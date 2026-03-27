//! macOS menu bar — App + Edit + Window
//! Provides native Cmd+C/V/X/A, Preferences, Quit

use objc2::runtime::Sel;
use objc2::{sel, msg_send};
use objc2_app_kit::{NSApplication, NSMenu, NSMenuItem, NSEventModifierFlags};
use objc2_foundation::{MainThreadMarker, NSString};

pub fn setup_main_menu(app: &NSApplication, mtm: MainThreadMarker) {
    let menu_bar = NSMenu::new(mtm);

    // ── App menu ─────────────────────────────────────────────────────────
    let app_item = empty_item(mtm);
    let app_menu = NSMenu::new(mtm);

    app_menu.addItem(&menu_item(mtm, "A propos de Software Factory", sel!(orderFrontStandardAboutPanel:), "", NSEventModifierFlags(0)));
    app_menu.addItem(&NSMenuItem::separatorItem(mtm));
    app_menu.addItem(&menu_item(mtm, "Preferences…", sel!(showPreferences:), ",", NSEventModifierFlags::NSEventModifierFlagCommand));
    app_menu.addItem(&NSMenuItem::separatorItem(mtm));
    let services_item = menu_item(mtm, "Services", sel!(invalid:), "", NSEventModifierFlags(0));
    let services_menu = NSMenu::new(mtm);
    services_item.setSubmenu(Some(&services_menu));
    unsafe { app.setServicesMenu(Some(&services_menu)); }
    app_menu.addItem(&services_item);
    app_menu.addItem(&NSMenuItem::separatorItem(mtm));
    app_menu.addItem(&menu_item(mtm, "Masquer Software Factory", sel!(hide:), "h", NSEventModifierFlags::NSEventModifierFlagCommand));
    app_menu.addItem(&menu_item(mtm, "Masquer les autres", sel!(hideOtherApplications:), "h",
        NSEventModifierFlags(NSEventModifierFlags::NSEventModifierFlagCommand.0 | NSEventModifierFlags::NSEventModifierFlagOption.0)));
    app_menu.addItem(&menu_item(mtm, "Tout afficher", sel!(unhideAllApplications:), "", NSEventModifierFlags(0)));
    app_menu.addItem(&NSMenuItem::separatorItem(mtm));
    app_menu.addItem(&menu_item(mtm, "Quitter Software Factory", sel!(terminate:), "q", NSEventModifierFlags::NSEventModifierFlagCommand));

    app_item.setSubmenu(Some(&app_menu));
    menu_bar.addItem(&app_item);

    // ── File menu ─────────────────────────────────────────────────────────
    let file_item = titled_item(mtm, "Fichier");
    let file_menu = NSMenu::new(mtm);
    file_menu.addItem(&menu_item(mtm, "Nouvelle session", sel!(newSession:), "n", NSEventModifierFlags::NSEventModifierFlagCommand));
    file_menu.addItem(&menu_item(mtm, "Nouvelle mission", sel!(newMission:), "N",
        NSEventModifierFlags(NSEventModifierFlags::NSEventModifierFlagCommand.0 | NSEventModifierFlags::NSEventModifierFlagShift.0)));
    file_menu.addItem(&NSMenuItem::separatorItem(mtm));
    file_menu.addItem(&menu_item(mtm, "Fermer", sel!(performClose:), "w", NSEventModifierFlags::NSEventModifierFlagCommand));
    file_item.setSubmenu(Some(&file_menu));
    menu_bar.addItem(&file_item);

    // ── Edit menu ─────────────────────────────────────────────────────────
    // WKWebView handles copy/paste via responder chain — these items activate it
    let edit_item = titled_item(mtm, "Edition");
    let edit_menu = unsafe { NSMenu::initWithTitle(NSMenu::alloc(), &NSString::from_str("Edition")) };
    edit_menu.addItem(&menu_item(mtm, "Annuler", sel!(undo:), "z", NSEventModifierFlags::NSEventModifierFlagCommand));
    edit_menu.addItem(&menu_item(mtm, "Rétablir", sel!(redo:), "Z",
        NSEventModifierFlags(NSEventModifierFlags::NSEventModifierFlagCommand.0 | NSEventModifierFlags::NSEventModifierFlagShift.0)));
    edit_menu.addItem(&NSMenuItem::separatorItem(mtm));
    edit_menu.addItem(&menu_item(mtm, "Couper", sel!(cut:), "x", NSEventModifierFlags::NSEventModifierFlagCommand));
    edit_menu.addItem(&menu_item(mtm, "Copier", sel!(copy:), "c", NSEventModifierFlags::NSEventModifierFlagCommand));
    edit_menu.addItem(&menu_item(mtm, "Coller", sel!(paste:), "v", NSEventModifierFlags::NSEventModifierFlagCommand));
    edit_menu.addItem(&menu_item(mtm, "Tout sélectionner", sel!(selectAll:), "a", NSEventModifierFlags::NSEventModifierFlagCommand));
    edit_menu.addItem(&NSMenuItem::separatorItem(mtm));
    edit_menu.addItem(&menu_item(mtm, "Rechercher…", sel!(find:), "f", NSEventModifierFlags::NSEventModifierFlagCommand));
    edit_item.setSubmenu(Some(&edit_menu));
    menu_bar.addItem(&edit_item);

    // ── Window menu ────────────────────────────────────────────────────────
    let win_item = titled_item(mtm, "Fenêtre");
    let win_menu = unsafe { NSMenu::initWithTitle(NSMenu::alloc(), &NSString::from_str("Fenêtre")) };
    win_menu.addItem(&menu_item(mtm, "Réduire", sel!(miniaturize:), "m", NSEventModifierFlags::NSEventModifierFlagCommand));
    win_menu.addItem(&menu_item(mtm, "Zoom", sel!(zoom:), "", NSEventModifierFlags(0)));
    win_menu.addItem(&NSMenuItem::separatorItem(mtm));
    win_menu.addItem(&menu_item(mtm, "Mettre au premier plan", sel!(makeKeyAndOrderFront:), "", NSEventModifierFlags(0)));
    unsafe { app.setWindowsMenu(Some(&win_menu)); }
    win_item.setSubmenu(Some(&win_menu));
    menu_bar.addItem(&win_item);

    app.setMainMenu(Some(&menu_bar));
}

// ── Helpers ───────────────────────────────────────────────────────────────

fn menu_item(mtm: MainThreadMarker, title: &str, action: Sel, key: &str, mods: NSEventModifierFlags) -> objc2::rc::Retained<NSMenuItem> {
    let item = unsafe {
        NSMenuItem::initWithTitle_action_keyEquivalent(
            NSMenuItem::alloc(),
            &NSString::from_str(title),
            Some(action),
            &NSString::from_str(key),
        )
    };
    item.setKeyEquivalentModifierMask(mods);
    item
}

fn empty_item(mtm: MainThreadMarker) -> objc2::rc::Retained<NSMenuItem> {
    NSMenuItem::separatorItem(mtm)  // Placeholder — will get submenu
}

fn titled_item(mtm: MainThreadMarker, title: &str) -> objc2::rc::Retained<NSMenuItem> {
    unsafe {
        NSMenuItem::initWithTitle_action_keyEquivalent(
            NSMenuItem::alloc(),
            &NSString::from_str(title),
            None,
            &NSString::from_str(""),
        )
    }
}
