#[cfg(feature = "desktop-gtk")]
fn main() {
    clipboard_history::desktop::gtk::run_application();
}

#[cfg(not(feature = "desktop-gtk"))]
fn main() {
    println!("clipboard-history daemon placeholder");
}
