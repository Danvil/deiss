use crate::{painter::*, utils::*};

/// Default guid
pub fn deiss_gui(ctx: &egui::Context, settings: &mut Settings, globals: &mut Globals) {
    egui::Window::new("DEISS").resizable(true).vscroll(true).default_open(true).show(ctx, |ui| {
        egui::CollapsingHeader::new("Mode Selection")
            .default_open(true)
            .show(ui, |ui| mode_prefs_gui(ui, &mut settings.mode_prefs));
        egui::CollapsingHeader::new("Waveform Selection")
            .default_open(true)
            .show(ui, |ui| waveform_prefs_gui(ui, &mut settings.waveform_prefs));
        egui::CollapsingHeader::new("Detail").default_open(true).show(ui, |ui| {
            egui::CollapsingHeader::new("GF")
                .default_open(true)
                .show(ui, |ui| settings_gf_gui(ui, &mut settings.gf, &mut globals.rand));
        });
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

/// GUI to change waveform selection preferences
fn waveform_prefs_gui(ui: &mut egui::Ui, waveform_prefs: &mut WaveformPrefs) {
    let current_priority = waveform_prefs.priority();
    let mut new_priority = current_priority;

    ui.label("Select priority waveform (leave unchecked for random selection):");
    ui.separator();

    for waveform_id in 1..=7 {
        let wid = WaveformId(waveform_id);
        ui.horizontal(|ui| {
            let is_current_waveform = current_priority == Some(wid);

            // Priority checkbox
            let mut is_priority = is_current_waveform;
            let checkbox = ui.checkbox(&mut is_priority, "");
            if checkbox.clicked() {
                if is_priority {
                    new_priority = Some(wid);
                } else if is_current_waveform {
                    new_priority = None;
                }
            }

            // Waveform label
            let label_color = if is_priority {
                egui::Color32::from_rgb(255, 200, 100)
            } else {
                ui.style().visuals.text_color()
            };
            ui.colored_label(label_color, format!("Waveform {}", waveform_id));
        });
    }

    // Apply priority change
    if new_priority != current_priority {
        waveform_prefs.set_priority(new_priority);
    }
}

/// Show GF values and allow to recompute
fn settings_gf_gui(ui: &mut egui::Ui, gf: &mut [f32; 6], rand: &mut Minstd) {
    ui.label("Nuclid color 'F':");
    ui.separator();

    // Display each GF value with a slider
    for (i, gf_val) in gf.iter_mut().enumerate() {
        ui.horizontal(|ui| {
            ui.label(format!("F[{}]:", i));
            ui.add(egui::Slider::new(gf_val, 0.0..=0.1).fixed_decimals(4).step_by(0.0001));
        });
    }

    ui.separator();

    // Button to regenerate random GF values
    if ui.button("Regenerate Random").clicked() {
        *gf = generate_gf(rand);
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
