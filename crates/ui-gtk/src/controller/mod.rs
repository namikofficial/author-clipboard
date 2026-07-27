//! Keyboard controllers shared across popup and manager.

pub mod focus;
pub mod key;
pub mod search;

use std::cell::RefCell;
use std::rc::Rc;

/// Install page-level key forwarding for the clipboard page.
///
/// These keys need access to the page's filter/refresh context. They are
/// forwarded from the window-level controller via the `on_page_key` callback.
/// This is shared between popup and manager.
pub fn install_page_keys(
    state: &Rc<RefCell<crate::app::AppState>>,
    effects_tx: &std::sync::mpsc::Sender<crate::Effect>,
) -> Box<dyn Fn(gtk4::gdk::Key, gtk4::gdk::ModifierType) -> gtk4::glib::Propagation> {
    use gtk4::glib::Propagation;
    let st = state.clone();
    let tx = effects_tx.clone();
    Box::new(move |key, mods| {
        let ctrl = mods.contains(gtk4::gdk::ModifierType::CONTROL_MASK);
        let shift = mods.contains(gtk4::gdk::ModifierType::SHIFT_MASK);

        // Ctrl+Shift+P: toggle pinned filter
        if ctrl && shift && key == gtk4::gdk::Key::p {
            let mut s = st.borrow_mut();
            s.filter = if s.filter == crate::PickerFilter::Pinned {
                crate::PickerFilter::All
            } else {
                crate::PickerFilter::Pinned
            };
            drop(s);
            let _ = tx.send(crate::Effect::RefreshItems);
            return Propagation::Stop;
        }

        // Ctrl+Shift+A: toggle starred filter
        if ctrl && shift && key == gtk4::gdk::Key::a {
            let mut s = st.borrow_mut();
            s.filter = if s.filter == crate::PickerFilter::Starred {
                crate::PickerFilter::All
            } else {
                crate::PickerFilter::Starred
            };
            drop(s);
            let _ = tx.send(crate::Effect::RefreshItems);
            return Propagation::Stop;
        }

        // Ctrl+Shift+C: collection chooser
        if ctrl && shift && key == gtk4::gdk::Key::c {
            let s = st.borrow();
            let selected_id = s.selected_id;
            drop(s);
            if let Some(id) = selected_id {
                let _ = tx.send(crate::Effect::ShowCollectionChooser(id));
            }
            return Propagation::Stop;
        }

        Propagation::Proceed
    })
}
