use vizia::prelude::*;
use crate::AppData;

pub struct TransportControls;

impl View for TransportControls {
    fn element(&self) -> Option<&'static str> {
        Some("transport-controls")
    }

    fn build(&self, cx: &mut Context) {
        HStack::new(cx, |cx| {
            // Record button
            Button::new(cx, |cx| {
                Label::new(cx, "●")
                    .width(Pixels(30))
                    .height(Pixels(30))
                    .background_color(Color::rgb(1.0, 0.0, 0.0))
                    .border_radius(Pixels(15))
            })
            .tooltip(cx, "Record")
            .on_press(|cx| {
                cx.emit(AppEvent::PlaybackStateChanged(2)); // 2 = record
            })
            .width(Pixels(40))
            .height(Pixels(40));

            Spacer::new(cx).width(Pixels(10));

            // Play button
            Button::new(cx, |cx| {
                Label::new(cx, "▶")
                    .width(Pixels(30))
                    .height(Pixels(30))
                    .background_color(Color::rgb(0.0, 1.0, 0.0))
                    .border_radius(Pixels(15))
            })
            .tooltip(cx, "Play")
            .on_press(|cx| {
                cx.emit(AppEvent::PlaybackStateChanged(1)); // 1 = play
            })
            .width(Pixels(40))
            .height(Pixels(40));

            Spacer::new(cx).width(Pixels(10));

            // Pause button
            Button::new(cx, |cx| {
                Label::new(cx, "❚❚")
                    .width(Pixels(30))
                    .height(Pixels(30))
                    .background_color(Color::rgb(1.0, 1.0, 0.0))
                    .border_radius(Pixels(15))
            })
            .tooltip(cx, "Pause")
            .on_press(|cx| {
                cx.emit(AppEvent::PlaybackStateChanged(3)); // 3 = pause
            })
            .width(Pixels(40))
            .height(Pixels(40));

            Spacer::new(cx).width(Pixels(10));

            // Stop button
            Button::new(cx, |cx| {
                Label::new(cx, "■")
                    .width(Pixels(30))
                    .height(Pixels(30))
                    .background_color(Color::rgb(1.0, 0.5, 0.0))
                    .border_radius(Pixels(15))
            })
            .tooltip(cx, "Stop")
            .on_press(|cx| {
                cx.emit(AppEvent::PlaybackStateChanged(0)); // 0 = stop
            })
            .width(Pixels(40))
            .height(Pixels(40));
        })
        .child_top(Stretch(1.0))
        .child_bottom(Stretch(1.0))
        .child_left(Stretch(1.0))
        .child_right(Stretch(1.0));
    }
}

// Custom events for the app
#[derive(Debug, Clone, Data)]
enum AppEvent {
    PlaybackStateChanged(i32),
}