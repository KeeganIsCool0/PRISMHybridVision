use vizia::prelude::*;
use crate::AppData;

pub struct AutomationLanes;

impl View for AutomationLanes {
    fn element(&self) -> Option<&'static str> {
        Some("automation-lanes")
    }

    fn build(&self, cx: &mut Context) {
        VStack::new(cx, |cx| {
            Label::new(cx, "Automation")
                .width(Pixels(300))
                .height(Pixels(30))
                .background_color(Color::rgb(0.2, 0.2, 0.2))
                .color(Color::white)
                .child_top(Stretch(1.0))
                .child_bottom(Stretch(1.0))
                .child_left(Stretch(1.0))
                .child_right(Stretch(1.0));

            // Placeholder for automation lanes
            VStack::new(cx, |cx| {
                // Create a few automation lanes
                for lane in 0..4 {
                    HStack::new(cx, |cx| {
                        // Lane name
                        Label::new(cx, match lane {
                            0 => "Master Vol",
                            1 => "Master Pan",
                            2 => "SG1 Vol",
                            3 => "SG2 Vol",
                            _ => "Lane",
                        })
                        .width(Pixels(80))
                        .height(Pixels(25))
                        .background_color(Color::rgb(0.15, 0.15, 0.15))
                        .color(Color::white)
                        .child_top(Stretch(1.0))
                        .child_bottom(Stretch(1.0))
                        .child_left(Stretch(1.0))
                        .child_right(Stretch(1.0));

                        // Automation curve (simplified as a line)
                        Canvas::new(cx, |cx| {
                            // Draw a simple line for automation
                            let points = [
                                (10.0, 50.0),
                                (50.0, 30.0),
                                (100.0, 60.0),
                                (150.0, 40.0),
                                (200.0, 70.0),
                            ];
                            for i in 0..points.len()-1 {
                                Line::new(cx)
                                    .start(points[i])
                                    .end(points[i+1])
                                    .stroke_width(Pixels(2.0))
                                    .stroke_color(Color::rgb(0.0, 0.8, 0.8));
                            }
                        })
                        .width(Pixels(200))
                        .height(Pixels(80))
                        .background_color(Color::rgb(0.1, 0.1, 0.1));
                    })
                    .height(Pixels(90))
                    .child_top(Stretch(1.0))
                    .child_bottom(Stretch(1.0))
                    .child_left(Stretch(1.0))
                    .child_right(Stretch(1.0))
                    .background_color(Color::rgb(0.05, 0.05, 0.05));
                }
            })
            .flex(1.0);
        })
        .width(Pixels(300))
        .background_color(Color::rgb(0.05, 0.05, 0.05));
    }
}