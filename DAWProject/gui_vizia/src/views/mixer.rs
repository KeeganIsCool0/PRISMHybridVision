use vizia::prelude::*;
use crate::AppData;

pub struct MixerView;

impl View for MixerView {
    fn element(&self) -> Option<&'static str> {
        Some("mixer-view")
    }

    fn build(&self, cx: &mut Context) {
        VStack::new(cx, |cx| {
            Label::new(cx, "Mixer")
                .width(Pixels(300))
                .height(Pixels(30))
                .background_color(Color::rgb(0.2, 0.2, 0.2))
                .color(Color::white)
                .child_top(Stretch(1.0))
                .child_bottom(Stretch(1.0))
                .child_left(Stretch(1.0))
                .child_right(Stretch(1.0));

            // Mixer channels (sub-groups and master)
            HStack::new(cx, |cx| {
                // Sub-groups
                for i in 0..8 {
                    VStack::new(cx, |cx| {
                        Label::new(cx, format!("SG{}", i+1))
                            .width(Pixels(40))
                            .height(Pixels(20))
                            .background_color(Color::rgb(0.15, 0.15, 0.15))
                            .color(Color::white)
                            .child_top(Stretch(1.0))
                            .child_bottom(Stretch(1.0))
                            .child_left(Stretch(1.0))
                            .child_right(Stretch(1.0));

                        // Fader
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

                        Label::new(cx, "Vol")
                            .width(Pixels(40))
                            .height(Pixels(20))
                            .background_color(Color::rgb(0.1, 0.1, 0.1))
                            .color(Color::white)
                            .child_top(Stretch(1.0))
                            .child_bottom(Stretch(1.0))
                            .child_left(Stretch(1.0))
                            .child_right(Stretch(1.0));
                    })
                    .width(Pixels(40))
                }

                Spacer::new(cx).width(Pixels(20));

                // Master fader
                VStack::new(cx, |cx| {
                    Label::new(cx, "Master")
                        .width(Pixels(60))
                        .height(Pixels(20))
                        .background_color(Color::rgb(0.2, 0.2, 0.2))
                        .color(Color::white)
                        .child_top(Stretch(1.0))
                        .child_bottom(Stretch(1.0))
                        .child_left(Stretch(1.0))
                        .child_right(Stretch(1.0));

                    // Master fader (vertical)
                    VStack::new(cx, |cx| {
                        Label::new(cx, "∞")
                            .width(Pixels(30))
                            .height(Pixels(10))
                            .color(Color::white);
                        Slider::new(cx, /* data */, /* lens */)
                            .width(Pixels(30))
                            .height(Pixels(150))
                            .background_color(Color::rgb(0.4, 0.4, 0.4));
                        Label::new(cx, "0")
                            .width(Pixels(30))
                            .height(Pixels(10))
                            .color(Color::white);
                    })
                    .width(Pixels(30))
                    .height(Pixels(170));

                    Label::new(cx, "Vol")
                        .width(Pixels(60))
                        .height(Pixels(20))
                        .background_color(Color::rgb(0.1, 0.1, 0.1))
                        .color(Color::white)
                        .child_top(Stretch(1.0))
                        .child_bottom(Stretch(1.0))
                        .child_left(Stretch(1.0))
                        .child_right(Stretch(1.0));
                })
                .width(Pixels(60));
            })
            .height(Pixels(200));
        })
        .width(Pixels(400))
        .background_color(Color::rgb(0.05, 0.05, 0.05));
    }
}