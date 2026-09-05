use vizia::prelude::*;
use crate::AppData;
use cpal::HostTrait;

pub struct DeviceSelector;

impl View for DeviceSelector {
    fn element(&self) -> Option<&'static str> {
        Some("device-selector")
    }

    fn build(&self, cx: &mut Context) {
        // This would typically be a popup/modal window
        // For simplicity, we'll make it a visible panel when triggered
        VStack::new(cx, |cx| {
            Label::new(cx, "Audio Device Selection")
                .width(Pixels(300))
                .height(Pixels(40))
                .background_color(Color::rgb(0.2, 0.2, 0.2))
                .color(Color::white)
                .child_top(Stretch(1.0))
                .child_bottom(Stretch(1.0))
                .child_left(Stretch(1.0))
                .child_right(Stretch(1.0));

            Tabs::new(cx, |cx| {
                Tab::new(cx, |cx| {
                    Label::new(cx, "Input")
                }, |cx| {
                    VStack::new(cx, |cx| {
                        Label::new(cx, "Select Input Device")
                            .width(Pixels(280))
                            .height(Pixels(30))
                            .background_color(Color::rgb(0.15, 0.15, 0.15))
                            .color(Color::white)
                            .child_top(Stretch(1.0))
                            .child_bottom(Stretch(1.0))
                            .child_left(Stretch(1.0))
                            .child_right(Stretch(1.0));

                        // Get available input devices
                        let host = cpal::default_host();
                        let devices = host.input_devices().unwrap_or_default();
                        let mut device_names = Vec::new();
                        for device in devices {
                            if let Ok(name) = device.name() {
                                device_names.push(name);
                            }
                        }

                        // List box for devices
                        ListBox::new(cx, device_names.len(), move |cx, index| {
                            Label::new(cx, &device_names[index])
                                .width(Pixels(260))
                                .height(Pixels(25))
                                .color(Color::white)
                                .child_top(Stretch(1.0))
                                .child_bottom(Stretch(1.0))
                                .child_left(Stretch(1.0))
                                .child_right(Stretch(1.0))
                                .background_color(Color::rgb(0.1, 0.1, 0.1));
                        })
                        .height(Pixels(150))
                        .on_select(move |cx, index| {
                            if let Some(device_name) = device_names.get(index) {
                                cx.emit(DeviceSelectorEvent::InputSelected(device_name.clone()));
                            }
                        });

                        Spacer::new(cx).height(Pixels(10));

                        HStack::new(cx, |cx| {
                            Button::new(cx, |cx| {
                                Label::new(cx, "OK")
                            })
                            .width(Pixels(80))
                            .height(Pixels(30))
                            .background_color(Color::rgb(0.0, 0.6, 0.0))
                            .color(Color::white)
                            .on_press(|cx| {
                                // Would get selected value from listbox
                                cx.emit(DeviceSelectorEvent::Cancel); // For now just close
                            });

                            Spacer::new(cx).width(Pixels(10));

                            Button::new(cx, |cx| {
                                Label::new(cx, "Cancel")
                            })
                            .width(Pixels(80))
                            .height(Pixels(30))
                            .background_color(Color::rgb(0.6, 0.0, 0.0))
                            .color(Color::white)
                            .on_press(|cx| {
                                cx.emit(DeviceSelectorEvent::Cancel);
                            });
                        })
                    })
                })
                .width(Pixels(300))
                .height(Pixels(200));

                Tab::new(cx, |cx| {
                    Label::new(cx, "Output")
                }, |cx| {
                    VStack::new(cx, |cx| {
                        Label::new(cx, "Select Output Device")
                            .width(Pixels(280))
                            .height(Pixels(30))
                            .background_color(Color::rgb(0.15, 0.15, 0.15))
                            .color(Color::white)
                            .child_top(Stretch(1.0))
                            .child_bottom(Stretch(1.0))
                            .child_left(Stretch(1.0))
                            .child_right(Stretch(1.0));

                        // Get available output devices
                        let host = cpal::default_host();
                        let devices = host.output_devices().unwrap_or_default();
                        let mut device_names = Vec::new();
                        for device in devices {
                            if let Ok(name) = device.name() {
                                device_names.push(name);
                            }
                        }

                        // List box for devices
                        ListBox::new(cx, device_names.len(), move |cx, index| {
                            Label::new(cx, &device_names[index])
                                .width(Pixels(260))
                                .height(Pixels(25))
                                .color(Color::white)
                                .child_top(Stretch(1.0))
                                .child_bottom(Stretch(1.0))
                                .child_left(Stretch(1.0))
                                .child_right(Stretch(1.0))
                                .background_color(Color::rgb(0.1, 0.1, 0.1));
                        })
                        .height(Pixels(150))
                        .on_select(move |cx, index| {
                            if let Some(device_name) = device_names.get(index) {
                                cx.emit(DeviceSelectorEvent::OutputSelected(device_name.clone()));
                            }
                        });

                        Spacer::new(cx).height(Pixels(10));

                        HStack::new(cx, |cx| {
                            Button::new(cx, |cx| {
                                Label::new(cx, "OK")
                            })
                            .width(Pixels(80))
                            .height(Pixels(30))
                            .background_color(Color::rgb(0.0, 0.6, 0.0))
                            .color(Color::white)
                            .on_press(|cx| {
                                cx.emit(DeviceSelectorEvent::Cancel); // For now just close
                            });

                            Spacer::new(cx).width(Pixels(10));

                            Button::new(cx, |cx| {
                                Label::new(cx, "Cancel")
                            })
                            .width(Pixels(80))
                            .height(Pixels(30))
                            .background_color(Color::rgb(0.6, 0.0, 0.0))
                            .color(Color::white)
                            .on_press(|cx| {
                                cx.emit(DeviceSelectorEvent::Cancel);
                            });
                        })
                    })
                })
                .width(Pixels(300))
                .height(Pixels(200));
            })
            .height(Pixels(250));
        })
        .width(Pixels(300))
        .background_color(Color::rgb(0.05, 0.05, 0.05))
        .border_color(Color::white)
        .border_width(Pixels(2));
    }
}