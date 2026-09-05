use vizia::prelude::*;
use crate::AppData;

pub struct TimeDisplay;

impl View for TimeDisplay {
    fn element(&self) -> Option<&'static str> {
        Some("time-display")
    }

    fn build(&self, cx: &mut Context) {
        Label::new(cx, || {
            // Format time as HH:MM:SS.mmm
            // In a real implementation, this would come from audio engine position
            let hours = 0;
            let minutes = 0;
            let seconds = 0;
            let milliseconds = 0;
            format!("{:02}:{:02}:{:02}.{:03}", hours, minutes, seconds, milliseconds)
        })
        .width(Pixels(150))
        .height(Pixels(30))
        .child_top(Stretch(1.0))
        .child_bottom(Stretch(1.0))
        .child_left(Stretch(1.0))
        .child_right(Stretch(1.0))
        .background_color(Color::rgb(0.1, 0.1, 0.1))
        .border_radius(Pixels(4))
        .color(Color::white);
    }
}