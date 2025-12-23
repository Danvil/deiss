use crate::painter::{ModePrefs, Settings};

/// Default guid
pub fn deiss_gui(ctx: &egui::Context, settings: &mut Settings) {
    egui::Window::new("DEISS").resizable(true).vscroll(true).default_open(true).show(ctx, |ui| {
        egui::CollapsingHeader::new("Mode Selection")
            .default_open(true)
            .show(ui, |ui| mode_prefs_gui(ui, &mut settings.mode_prefs));
    });
}

/// GUI to change mode selection preferences
fn mode_prefs_gui(ui: &mut egui::Ui, mode_prefs: &mut ModePrefs) {
    let current_priority = mode_prefs.priority();
    let mut new_priority = current_priority;

    for (mode_id, weight) in mode_prefs.weights_mut().iter_mut() {
        ui.horizontal(|ui| {
            let is_current_mode = current_priority == Some(*mode_id);
            let has_priority_set = current_priority.is_some();
            let is_enabled = !has_priority_set || is_current_mode;

            // Priority checkbox
            let mut is_priority = is_current_mode;
            let checkbox = ui.add_enabled(is_enabled, egui::Checkbox::new(&mut is_priority, ""));
            if checkbox.clicked() && is_enabled {
                if is_priority {
                    new_priority = Some(*mode_id);
                } else if is_current_mode {
                    new_priority = None;
                }
            }

            // Mode label
            let label_color = if is_enabled {
                if is_priority {
                    egui::Color32::from_rgb(255, 200, 100)
                } else {
                    ui.style().visuals.text_color()
                }
            } else {
                ui.style().visuals.weak_text_color()
            };
            ui.colored_label(label_color, format!("Mode {}", mode_id.0));

            // Slider to change weight (0-5)
            let mut stars = *weight as i32;
            let slider = egui::Slider::new(&mut stars, 0..=5).show_value(false);

            if is_enabled {
                ui.add(slider);
                *weight = stars as u32;
            } else {
                ui.add_enabled(false, slider);
            }

            // Display of weight as stars
            let star_text = "★".repeat(*weight as usize) + &"☆".repeat((5 - *weight) as usize);
            ui.colored_label(label_color, star_text);
        });
    }

    // Apply priority change
    if new_priority != current_priority {
        mode_prefs.set_priority(new_priority);
    }
}

/// Test GUI
pub fn demo_gui(ctx: &egui::Context) {
    egui::Window::new("winit + egui + wgpu says hello!")
        .resizable(true)
        .vscroll(true)
        .default_open(false)
        .show(ctx, |ui| {
            ui.label("Label!");

            if ui.button("Button!").clicked() {
                println!("boom!")
            }
        });
}
