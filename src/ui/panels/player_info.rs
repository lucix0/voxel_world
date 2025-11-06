use egui::{Color32, RichText, Ui};
use cgmath::Point3;

pub struct PlayerInfoPanel;

impl PlayerInfoPanel {
    pub fn show(
        ui: &mut Ui,
        position: Point3<f32>,
        velocity: cgmath::Vector3<f32>,
    ) {
        ui.heading(RichText::new("Player Info").color(Color32::WHITE));
        ui.separator();

        ui.colored_label(
            egui::Color32::WHITE,
            format!(
                "Position: {:.2}, {:.2}, {:.2}",
                position.x,
                position.y,
                position.z
            )
        );
        ui.colored_label(
            egui::Color32::WHITE,
            format!(
                "Velocity: {:.2}, {:.2}, {:.2}",
                velocity.x,
                velocity.y,
                velocity.z
            )
        );
    }
}