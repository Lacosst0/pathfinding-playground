use bevy::prelude::*;

pub const HOVERED: f32 = 0.01;
pub const PRESSED: f32 = 0.1;

pub mod button;
pub mod checkbox;
pub mod separator;
pub mod slider;
pub mod text;

pub use button::*;
pub use checkbox::*;
pub use separator::*;
pub use slider::*;
pub use text::*;

pub struct ComponentsPlugin;
impl Plugin for ComponentsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, update_button_style)
            .add_systems(Update, update_slider_style)
            .add_systems(Update, update_checkbox_style);
    }
}
