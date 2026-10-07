use objc2::runtime::Sel;
use objc2::{MainThreadMarker, MainThreadOnly, sel};
use objc2_app_kit::{NSApplication, NSMenu, NSMenuItem};
use objc2_foundation::{NSString, ns_string};

/// Adds the Edit menu to the menu bar: the text fields of native dialogs take
/// Cut, Copy, Paste and Select All from its key equivalents.
pub fn install_edit_menu() {
    let Some(main_thread) = MainThreadMarker::new() else {
        return;
    };
    let Some(menu_bar) = NSApplication::sharedApplication(main_thread).mainMenu() else {
        return;
    };
    let edit_menu = NSMenu::initWithTitle(NSMenu::alloc(main_thread), ns_string!("Edit"));
    let commands = [
        (ns_string!("Cut"), sel!(cut:), ns_string!("x")),
        (ns_string!("Copy"), sel!(copy:), ns_string!("c")),
        (ns_string!("Paste"), sel!(paste:), ns_string!("v")),
        (ns_string!("Select All"), sel!(selectAll:), ns_string!("a")),
    ];
    for (title, action, key) in commands {
        edit_menu.addItem(&text_command_item(main_thread, title, action, key));
    }
    let edit_entry = NSMenuItem::new(main_thread);
    edit_entry.setSubmenu(Some(&edit_menu));
    menu_bar.addItem(&edit_entry);
}

fn text_command_item(
    main_thread: MainThreadMarker,
    title: &NSString,
    action: Sel,
    key: &NSString,
) -> objc2::rc::Retained<NSMenuItem> {
    // SAFETY: the action is a standard responder selector taking the sender as its only argument.
    unsafe {
        NSMenuItem::initWithTitle_action_keyEquivalent(
            NSMenuItem::alloc(main_thread),
            title,
            Some(action),
            key,
        )
    }
}
