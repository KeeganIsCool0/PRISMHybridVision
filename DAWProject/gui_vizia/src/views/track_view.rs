use vizia::prelude::*;
use crate::AppData;

pub struct TrackView;

impl View for TrackView {
    fn element(&self) -> Option<&'static str> {
        Some("track-view")
    }

    fn build(&self, cx: &mut Context) {
        VStack::new(cx, |cx| {
            Label::new(cx, "Tracks")
                .width(Pixels(300))
                .height(Pixels(30))
                .background_color(Color::rgb(0.2, 0.2, 0.2))
                .color(Color::white)
                .child_top(Stretch(1.0))
                .child_bottom(Stretch(1.0))
                .child_left(Stretch(1.0))
                .child_right(Stretch(1.0));

            // Placeholder for track list
            VStack::new(cx, |cx| {
                for i in 0..8 {
                    HStack::new(cx, |cx| {
                        // Track number
                        Label::new(cx, format!("{}", i + 1))
                            .width(Pixels(30))
                            .height(Pixels(25))
                            .background_color(Color::rgb(0.15, 0.15, 0.15))
                            .color(Color::white)
                            .child_top(Stretch(1.0))
                            .child_bottom(Stretch(1.0))
                            .child_left(Stretch(1.0))
                            .child_right(Stretch(1.0));

                        // Record arm button
                        Toggle::new(cx, |cx| {
                            // We'll bind to track_record_arm later
                        })
                        .width(Pixels(25))
                        .height(Pixels(25))
                        .background_color(Color::rgb(0.8, 0.0, 0.0)) // Red when armed
                        .border_color(Color::white)
                        .border_width(Pixels(1));

                        Spacer::new(cx).width(Pixels(5));

                        // Track name
                        Label::new(cx, format!("Track {}", i + 1))
                            .width(Pixels(100))
                            .height(Pixels(25))
                            .color(Color::white)
                            .child_top(Stretch(1.0))
                            .child_bottom(Stretch(1.0))
                            .child_left(Stretch(1.0))
                            .child_right(Stretch(1.0));

                        Spacer::new(cx);

                        // Fader (vertical slider)
                        VStack::new(cx, |cx| {
                            Label::new(cx, "∞")
                                .width(Pixels(20))
                                .height(Pixels(10))
                                .color(Color::white);
                            Slider::new(cx, /* data */, /* lens */)
                                .width(Pixels(20))
                                .height(Pixels(100))
                                .background_color(Color::rgb(0.3, 0.3, 0.3));
                            Label::new(cx, "0")
                                .width(Pixels(20))
                                .height(Pixels(10))
                                .color(Color::white);
                        })
                        .width(Pixels(20))
                        .height(Pixels(130));

                        Spacer::new(cx).width(Pixels(5));

                        // Pan knob (placeholder)
                        Label::new(cx, "Pan")
                            .width(Pixels(40))
                            .height(Pixels(25))
                            .background_color(Color::rgb(0.2, 0.2, 0.2))
                            .color(Color::white)
                            .child_top(Stretch(1.0))
                            .child_bottom(Stretch(1.0))
                            .child_left(Stretch(1.0))
                            .child_right(Stretch(1.0));
                    })
                    .height(Pixels(30))
                    .child_top(Stretch(1.0))
                    .child_bottom(Stretch(1.0))
                    .child_left(Stretch(1.0))
                    .child_right(Stretch(1.0))
                    .background_color(Color::rgb(0.1, 0.1, 0.1));
                }
            })
            .flex(1.0);
        })
        .width(Pixels(300))
        .background_color(Color::rgb(0.05, 0.05, 0.05));
    }
}