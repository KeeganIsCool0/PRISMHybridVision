use vizia::prelude::*;
use crate::AppData;

pub struct SpatialPanner;

impl View for SpatialPanner {
    fn element(&self) -> Option<&'static str> {
        Some("spatial-panner")
    }

    fn build(&self, cx: &mut Context) {
        VStack::new(cx, |cx| {
            Label::new(cx, "9.2.6 Spatial Panner")
                .width(Pixels(300))
                .height(Pixels(30))
                .background_color(Color::rgb(0.2, 0.2, 0.2))
                .color(Color::white)
                .child_top(Stretch(1.0))
                .child_bottom(Stretch(1.0))
                .child_left(Stretch(1.0))
                .child_right(Stretch(1.0));

            // Canvas for speaker visualization
            Canvas::new(cx, |cx| {
                // Draw speaker positions for 9.2.6 configuration
                // This is a simplified representation

                // Center speaker
                Circle::new(cx)
                    .position((150.0, 100.0))
                    .radius(Pixels(10.0))
                    .color(Color::rgb(0.8, 0.8, 0.2));
                Label::new(cx, "C")
                    .position((145.0, 95.0))
                    .color(Color::black);

                // Left front
                Circle::new(cx)
                    .position((80.0, 100.0))
                    .radius(Pixels(8.0))
                    .color(Color::rgb(0.2, 0.6, 0.8));
                Label::new(cx, "L")
                    .position((75.0, 95.0))
                    .color(Color::white);

                // Right front
                Circle::new(cx)
                    .position((220.0, 100.0))
                    .radius(Pixels(8.0))
                    .color(Color::rgb(0.2, 0.6, 0.8));
                Label::new(cx, "R")
                    .position((215.0, 95.0))
                    .color(Color::white);

                // Left surround
                Circle::new(cx)
                    .position((50.0, 180.0))
                    .radius(Pixels(6.0))
                    .color(Color::rgb(0.2, 0.6, 0.8));
                Label::new(cx, "Ls")
                    .position((45.0, 175.0))
                    .color(Color::white);

                // Right surround
                Circle::new(cx)
                    .position((250.0, 180.0))
                    .radius(Pixels(6.0))
                    .color(Color::rgb(0.2, 0.6, 0.8));
                Label::new(cx, "Rs")
                    .position((245.0, 175.0))
                    .color(Color::white);

                // Height speakers (top layer)
                Circle::new(cx)
                    .position((100.0, 50.0))
                    .radius(Pixels(6.0))
                    .color(Color::rgb(0.8, 0.2, 0.6));
                Label::new(cx, "Lt")
                    .position((95.0, 45.0))
                    .color(Color::white);

                Circle::new(cx)
                    .position((200.0, 50.0))
                    .radius(Pixels(6.0))
                    .color(Color::rgb(0.8, 0.2, 0.6));
                Label::new(cx, "Rt")
                    .position((195.0, 45.0))
                    .color(Color::white);

                // Top middle
                Circle::new(cx)
                    .position((150.0, 30.0))
                    .radius(Pixels(6.0))
                    .color(Color::rgb(0.8, 0.2, 0.6));
                Label::new(cx, "T")
                    .position((145.0, 25.0))
                    .color(Color::white);

                // Bottom layer (for 9.2.6 - typically 2 bottom front)
                Circle::new(cx)
                    .position((100.0, 150.0))
                    .radius(Pixels(6.0))
                    .color(Color::rgb(0.6, 0.2, 0.8));
                Label::new(cx, "Lb")
                    .position((95.0, 145.0))
                    .color(Color::white);

                Circle::new(cx)
                    .position((200.0, 150.0))
                    .radius(Pixels(6.0))
                    .color(Color::rgb(0.6, 0.2, 0.8));
                Label::new(cx, "Rb")
                    .position((195.0, 145.0))
                    .color(Color::white);

                // VU meters (left and right)
                // Left VU meter
                Rectangle::new(cx)
                    .position((20.0, 50.0))
                    .size((20.0, 120.0))
                    .color(Color::rgb(0.1, 0.1, 0.1));
                for i in 0..12 {
                    let level = if i < 8 { 1.0 } else { 0.3 }; // Simulate some level
                    let color = if i < 6 {
                        Color::rgb(0.0, 1.0 - level * 0.3, 0.0) // Green to yellow
                    } else if i < 10 {
                        Color::rgb(1.0, level * 0.5, 0.0) // Yellow to red
                    } else {
                        Color::rgb(1.0, 0.0, 0.0) // Red
                    };
                    Rectangle::new(cx)
                        .position((22.0, 160.0 - (i * 10.0)))
                        .size((16.0, 8.0))
                        .color(color);
                }
                Label::new(cx, "L")
                    .position((15.0, 180.0))
                    .color(Color::white);

                // Right VU meter
                Rectangle::new(cx)
                    .position((260.0, 50.0))
                    .size((20.0, 120.0))
                    .color(Color::rgb(0.1, 0.1, 0.1));
                for i in 0..12 {
                    let level = if i < 6 { 1.0 } else { 0.4 }; // Different level for right
                    let color = if i < 6 {
                        Color::rgb(0.0, 1.0 - level * 0.3, 0.0)
                    } else if i < 10 {
                        Color::rgb(1.0, level * 0.5, 0.0)
                    } else {
                        Color::rgb(1.0, 0.0, 0.0)
                    };
                    Rectangle::new(cx)
                        .position((262.0, 160.0 - (i * 10.0)))
                        .size((16.0, 8.0))
                        .color(color);
                }
                Label::new(cx, "R")
                    .position((255.0, 180.0))
                    .color(Color::white);
            })
            .width(Pixels(300))
            .height(Pixels(200));
        })
        .width(Pixels(300))
        .background_color(Color::rgb(0.05, 0.05, 0.05));
    }
}