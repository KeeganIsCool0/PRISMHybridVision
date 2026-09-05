//! Standalone control surface for the Audio Console VST3.
use cpal::traits::{DeviceTrait, HostTrait};
use vizia::prelude::*;

const PANEL: Color = Color::rgb(12, 19, 25);
const HEADER: Color = Color::rgb(26, 43, 56);
const TEXT: Color = Color::rgb(215, 230, 235);
const CYAN: Color = Color::rgb(94, 210, 240);

fn device_name(input: bool) -> String {
    let host = cpal::default_host();
    let devices = if input {
        host.input_devices()
    } else {
        host.output_devices()
    };
    devices
        .ok()
        .and_then(|mut d| d.next())
        .and_then(|d| d.name().ok())
        .unwrap_or_else(|| "No device detected".into())
}

fn title(cx: &mut Context, text: &'static str) {
    Label::new(cx, text)
        .height(Pixels(25.0))
        .background_color(HEADER)
        .color(CYAN);
}

fn transport(cx: &mut Context, text: &'static str, color: Color) {
    Button::new(cx, move |cx| Label::new(cx, text).color(TEXT))
        .width(Pixels(68.0))
        .height(Pixels(30.0))
        .background_color(color);
}

/// Opens the standalone console. VST audio buffers are supplied by the host;
/// CPAL is used here for hardware discovery.
pub fn run() {
    let input = device_name(true);
    let output = device_name(false);
    let _ = Application::new(move |cx| {
        Window::new(cx, move |cx| {
            VStack::new(cx, |cx| {
                HStack::new(cx, |cx| {
                    Label::new(cx, "AUDIO CONSOLE • 17.9.2.6")
                        .width(Pixels(250.0))
                        .color(CYAN);
                    transport(cx, "● REC", Color::rgb(150, 20, 30));
                    transport(cx, "▶ PLAY", Color::rgb(15, 95, 55));
                    transport(cx, "Ⅱ PAUSE", Color::rgb(100, 80, 20));
                    transport(cx, "■ STOP", Color::rgb(55, 60, 65));
                    Label::new(cx, "01:00:00:00 | 48 kHz / 24-bit").color(TEXT);
                })
                .height(Pixels(42.0))
                .background_color(Color::rgb(5, 9, 14));
                HStack::new(cx, |cx| {
                    VStack::new(cx, |cx| {
                        title(cx, "SPATIAL PANNER • 17.9.2.6");
                        for (label, color) in [
                            ("TOP: TFL TFR TML TMR TRL TRR", Color::rgb(220, 100, 220)),
                            ("WIDE: Lw  L  C  R  Rw", CYAN),
                            ("SURR: Lss Ls    Rs Rss", CYAN),
                            ("LFE: LFE1  LFE2", Color::rgb(250, 190, 60)),
                            ("Selected object: ● Drag to pan", TEXT),
                            (
                                "L ███████░░ -8.2 dB   R ██████░░░ -10.1 dB",
                                Color::rgb(70, 230, 130),
                            ),
                        ] {
                            Label::new(cx, label).color(color);
                        }
                    })
                    .width(Pixels(480.0))
                    .height(Pixels(180.0))
                    .background_color(PANEL);
                    VStack::new(cx, |cx| {
                        title(cx, "AUDIO I/O • CPAL");
                        Label::new(cx, format!("Input: {input}")).color(TEXT);
                        Button::new(cx, |cx| Label::new(cx, "Select audio input").color(TEXT))
                            .height(Pixels(30.0))
                            .background_color(HEADER);
                        Label::new(cx, format!("Output: {output}")).color(TEXT);
                        Button::new(cx, |cx| Label::new(cx, "Select audio output").color(TEXT))
                            .height(Pixels(30.0))
                            .background_color(HEADER);
                    })
                    .width(Pixels(260.0))
                    .height(Pixels(180.0))
                    .background_color(PANEL);
                });
                VStack::new(cx, |cx| {
                    title(
                        cx,
                        "TIMECODE GRID • 01:00:00:00       01:00:10:00       01:00:20:00",
                    );
                    Label::new(
                        cx,
                        "│─────│─────│─────│─────│─────│─────│ TIME BAR / SCROLL VIEW",
                    )
                    .color(CYAN);
                    for track in [
                        "● 01 VOX LEAD  ╲╱╲╱╲╱╲╱╲╱╲╱",
                        "● 02 ROOM PAIR ╲╱╲╱╲╱╲╱╲╱╲╱",
                        "○ 03 MUSIC BED ╲╱╲╱╲╱╲╱╲╱╲╱",
                        "○ 04 FX RETURN ╲╱╲╱╲╱╲╱╲╱╲╱",
                    ] {
                        Label::new(cx, track)
                            .height(Pixels(25.0))
                            .color(CYAN)
                            .background_color(Color::rgb(14, 25, 33));
                    }
                })
                .height(Pixels(155.0))
                .background_color(PANEL);
                HStack::new(cx, |cx| {
                    VStack::new(cx, |cx| {
                        title(cx, "AUTOMATION");
                        Label::new(cx, "Master volume ───╱╲────╲────")
                            .color(Color::rgb(250, 170, 50));
                        Label::new(cx, "SG 1 volume   ───────╲╱──────")
                            .color(Color::rgb(250, 170, 50));
                    })
                    .width(Pixels(350.0))
                    .height(Pixels(130.0))
                    .background_color(PANEL);
                    VStack::new(cx, |cx| {
                        title(cx, "MIXER • 8 SUBGROUPS + MASTER");
                        HStack::new(cx, |cx| {
                            for channel in [
                                "SG1", "SG2", "SG3", "SG4", "SG5", "SG6", "SG7", "SG8", "MASTER",
                            ] {
                                VStack::new(cx, move |cx| {
                                    Label::new(cx, channel).color(TEXT);
                                    Label::new(cx, "│█│\n│█│\n│█│").color(if channel == "MASTER" {
                                        Color::rgb(250, 170, 50)
                                    } else {
                                        CYAN
                                    });
                                    Label::new(cx, "0.0").color(TEXT);
                                })
                                .width(Pixels(48.0));
                            }
                        });
                    })
                    .width(Pixels(480.0))
                    .height(Pixels(130.0))
                    .background_color(PANEL);
                });
            })
            .background_color(Color::rgb(4, 7, 10));
        });
    })
    .run();
}
