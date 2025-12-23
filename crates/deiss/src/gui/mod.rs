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
