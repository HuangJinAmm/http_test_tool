#[allow(clippy::ptr_arg)] // false positive
pub fn password_ui(ui: &mut egui::Ui, password: &mut String) -> egui::Response {
    // This widget has its own state — show or hide password characters (`show_plaintext`).
    // In this case we use a simple `bool`, but you can also declare your own type.
    // It must implement at least `Clone` and be `'static`.
    // If you use the `persistence` feature, it also must implement `serde::{Deserialize, Serialize}`.

    // Generate an id for the state
    let state_id = ui.id().with("show_plaintext");

    // Get state for this widget.
    // You should get state by value, not by reference to avoid borrowing of [`Memory`].
    let mut show_plaintext = ui.data_mut(|d| d.get_temp::<bool>(state_id).unwrap_or(false));

    // Process ui, change a local copy of the state
    // We want TextEdit to fill entire space, and have button after that, so in that case we can
    // change direction to right_to_left.
    let result = ui.horizontal(|ui| {
        // Toggle the `show_plaintext` bool with a button:
        // Show the password field:
        ui.add(egui::TextEdit::singleline(password).password(!show_plaintext));
        let response = ui
            .add(egui::SelectableLabel::new(show_plaintext, "👁"))
            .on_hover_text("显示/隐藏密码");

        if response.clicked() {
            show_plaintext = !show_plaintext;
        }
    });

    // Store the (possibly changed) state:
    ui.data_mut(|d| d.insert_temp(state_id, show_plaintext));

    // All done! Return the interaction response so the user can check what happened
    // (hovered, clicked, …) and maybe show a tooltip:
    result.response
}

// A wrapper that allows the more idiomatic usage pattern: `ui.add(…)`
/// Password entry field with ability to toggle character hiding.
///
/// ## Example:
/// ``` ignore
/// ui.add(password(&mut my_password));
/// ```
pub fn password(password: &mut String) -> impl egui::Widget + '_ {
    move |ui: &mut egui::Ui| password_ui(ui, password)
}
