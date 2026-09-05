use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use eframe::egui::{self, Color32, Pos2, Rect, Sense, Stroke, Vec2};
use hound::{SampleFormat, WavSpec, WavWriter};
use midir::{MidiOutput, MidiOutputConnection};
use std::{
    fs::File,
    io::BufWriter,
    path::PathBuf,
    sync::{
        atomic::{AtomicU32, Ordering},
        Arc, Mutex,
    },
    time::{Duration, Instant},
};

const CHANNEL_LAYOUT: [&str; 17] = [
    "L", "R", "C", "Lw", "Rw", "Ls", "Rs", "Lrs", "Rrs", "LFE 1", "LFE 2", "TFL", "TFR", "TML",
    "TMR", "TRL", "TRR",
];
const TRACK_NAMES: [&str; 8] = [
    "Vox Lead",
    "Room Pair",
    "Music Bed",
    "FX Return",
    "Guitar",
    "Keys",
    "Ambience",
    "Print",
];

#[derive(Clone, Copy, PartialEq, Eq)]
enum Transport {
    Stopped,
    Playing,
    Paused,
    Recording,
}
impl Transport {
    fn label(self) -> &'static str {
        match self {
            Self::Stopped => "STOPPED",
            Self::Playing => "PLAYING",
            Self::Paused => "PAUSED",
            Self::Recording => "RECORDING",
        }
    }
}

/// CPAL-backed input recorder. Each physical input channel becomes a mono WAV track.
struct AudioEngine {
    writers: Arc<Mutex<Vec<Option<WavWriter<BufWriter<File>>>>>>,
    peaks: Arc<[AtomicU32; 2]>,
    stream: Option<cpal::Stream>,
    track_paths: Vec<PathBuf>,
}
impl Default for AudioEngine {
    fn default() -> Self {
        Self {
            writers: Arc::new(Mutex::new(Vec::new())),
            peaks: Arc::new([AtomicU32::new(0), AtomicU32::new(0)]),
            stream: None,
            track_paths: Vec::new(),
        }
    }
}
impl AudioEngine {
    fn start(&mut self, device: cpal::Device) -> Result<usize, String> {
        self.stop();
        let supported = device.default_input_config().map_err(|e| e.to_string())?;
        let config: cpal::StreamConfig = supported.clone().into();
        let channels = config.channels as usize;
        std::fs::create_dir_all("recordings").map_err(|e| e.to_string())?;
        let spec = WavSpec {
            channels: 1,
            sample_rate: config.sample_rate.0,
            bits_per_sample: 32,
            sample_format: SampleFormat::Float,
        };
        let mut new_writers = Vec::with_capacity(channels);
        self.track_paths.clear();
        for channel in 0..channels {
            let path = PathBuf::from(format!("recordings/track_{channel}.wav"));
            new_writers.push(Some(
                WavWriter::create(&path, spec).map_err(|e| e.to_string())?,
            ));
            self.track_paths.push(path);
        }
        *self
            .writers
            .lock()
            .map_err(|_| "Audio writer lock failed")? = new_writers;
        let writers = Arc::clone(&self.writers);
        let peaks = Arc::clone(&self.peaks);
        let error = |err| eprintln!("CPAL input error: {err}");
        let stream = match supported.sample_format() {
            cpal::SampleFormat::F32 => device.build_input_stream(
                &config,
                move |data: &[f32], _| write_samples(data, channels, &writers, &peaks),
                error,
                None,
            ),
            cpal::SampleFormat::I16 => device.build_input_stream(
                &config,
                move |data: &[i16], _| {
                    let samples: Vec<f32> =
                        data.iter().map(|&s| s as f32 / i16::MAX as f32).collect();
                    write_samples(&samples, channels, &writers, &peaks);
                },
                error,
                None,
            ),
            cpal::SampleFormat::U16 => device.build_input_stream(
                &config,
                move |data: &[u16], _| {
                    let samples: Vec<f32> = data
                        .iter()
                        .map(|&s| (s as f32 / u16::MAX as f32) * 2.0 - 1.0)
                        .collect();
                    write_samples(&samples, channels, &writers, &peaks);
                },
                error,
                None,
            ),
            format => return Err(format!("Unsupported CPAL input format: {format:?}")),
        }
        .map_err(|e| e.to_string())?;
        stream.play().map_err(|e| e.to_string())?;
        self.stream = Some(stream);
        Ok(channels)
    }
    fn stop(&mut self) {
        self.stream.take();
        if let Ok(mut writers) = self.writers.lock() {
            for writer in writers.iter_mut() {
                *writer = None;
            }
        }
    }
    fn pause(&self) -> Result<(), String> {
        self.stream
            .as_ref()
            .map(cpal::Stream::pause)
            .transpose()
            .map_err(|e| e.to_string())
            .map(|_| ())
    }
    fn resume(&self) -> Result<(), String> {
        self.stream
            .as_ref()
            .map(cpal::Stream::play)
            .transpose()
            .map_err(|e| e.to_string())
            .map(|_| ())
    }
    fn peak(&self, side: usize) -> f32 {
        f32::from_bits(self.peaks[side].load(Ordering::Relaxed))
    }
}
fn write_samples(
    data: &[f32],
    channels: usize,
    writers: &Arc<Mutex<Vec<Option<WavWriter<BufWriter<File>>>>>>,
    peaks: &Arc<[AtomicU32; 2]>,
) {
    let mut peak = [0.0_f32; 2];
    if let Ok(mut writers) = writers.lock() {
        for frame in data.chunks(channels) {
            for (channel, sample) in frame.iter().copied().enumerate() {
                if channel < 2 {
                    peak[channel] = peak[channel].max(sample.abs());
                }
                if let Some(Some(writer)) = writers.get_mut(channel) {
                    let _ = writer.write_sample(sample);
                }
            }
        }
    }
    for side in 0..2 {
        peaks[side].store(peak[side].to_bits(), Ordering::Relaxed);
    }
}

struct DawApp {
    engine: AudioEngine,
    input_devices: Vec<cpal::Device>,
    input_names: Vec<String>,
    output_names: Vec<String>,
    selected_input: usize,
    selected_output: usize,
    midi_port_names: Vec<String>,
    selected_midi_port: usize,
    midi_out: Option<MidiOutputConnection>,
    transport: Transport,
    started_at: Option<Instant>,
    paused_elapsed: Duration,
    status: String,
    subgroup_levels: [f32; 8],
    master_level: f32,
    track_armed: [bool; 8],
    selected_speaker: usize,
}
impl Default for DawApp {
    fn default() -> Self {
        let host = cpal::default_host();
        let input_devices: Vec<_> = host
            .input_devices()
            .map(|d| d.collect())
            .unwrap_or_default();
        let input_names = input_devices
            .iter()
            .map(|d| d.name().unwrap_or_else(|_| "Unknown input".into()))
            .collect();
        let output_names = host
            .output_devices()
            .map(|d| d.filter_map(|d| d.name().ok()).collect())
            .unwrap_or_default();
        let midi_port_names = MidiOutput::new("Audio Console MMC")
            .map(|output| {
                output
                    .ports()
                    .iter()
                    .map(|port| {
                        output
                            .port_name(port)
                            .unwrap_or_else(|_| "Unnamed MIDI port".into())
                    })
                    .collect()
            })
            .unwrap_or_default();
        Self {
            engine: AudioEngine::default(),
            input_devices,
            input_names,
            output_names,
            selected_input: 0,
            selected_output: 0,
            midi_port_names,
            selected_midi_port: 0,
            midi_out: None,
            transport: Transport::Stopped,
            started_at: None,
            paused_elapsed: Duration::ZERO,
            status: "Ready — choose an input and arm tracks.".into(),
            subgroup_levels: [0.75; 8],
            master_level: 0.80,
            track_armed: [false; 8],
            selected_speaker: 2,
        }
    }
}
impl DawApp {
    /// Sends MIDI Machine Control to the DAW transport.  The selected DAW MIDI
    /// input must have MMC/remote control enabled by the user.
    fn send_mmc(&mut self, command: u8) -> Result<(), String> {
        if self.midi_out.is_none() {
            let output = MidiOutput::new("Audio Console MMC").map_err(|e| e.to_string())?;
            let ports = output.ports();
            let port = ports
                .get(self.selected_midi_port)
                .ok_or("No MIDI output port selected")?;
            self.midi_out = Some(
                output
                    .connect(port, "audio-console-mmc")
                    .map_err(|e| e.to_string())?,
            );
        }
        self.midi_out
            .as_mut()
            .ok_or("MIDI output is unavailable")?
            .send(&[0xF0, 0x7F, 0x7F, 0x06, command, 0xF7])
            .map_err(|e| e.to_string())
    }
    fn elapsed(&self) -> Duration {
        self.paused_elapsed
            + self
                .started_at
                .map(|time| time.elapsed())
                .unwrap_or_default()
    }
    fn set_transport(&mut self, state: Transport) {
        match state {
            Transport::Recording => {
                if let Some(device) = self.input_devices.get(self.selected_input).cloned() {
                    match self.engine.start(device) {
                        Ok(channels) => {
                            self.transport = state;
                            self.started_at = Some(Instant::now());
                            self.paused_elapsed = Duration::ZERO;
                            self.status = match self.send_mmc(0x06) {
                                Ok(()) => format!("Recording {channels} CPAL channels; sent MMC Record to DAW."),
                                Err(error) => format!("Recording {channels} CPAL channels locally; DAW MMC failed: {error}"),
                            };
                        }
                        Err(error) => self.status = format!("Could not start CPAL input: {error}"),
                    }
                } else {
                    self.status = "No CPAL input device is available.".into();
                }
            }
            Transport::Stopped => {
                self.engine.stop();
                self.transport = state;
                self.started_at = None;
                self.paused_elapsed = Duration::ZERO;
                self.status = match self.send_mmc(0x01) {
                    Ok(()) => "Stopped; sent MMC Stop to DAW.".into(),
                    Err(error) => format!("Stopped locally; DAW MMC failed: {error}"),
                };
            }
            Transport::Paused => {
                if let Some(start) = self.started_at.take() {
                    self.paused_elapsed += start.elapsed();
                }
                self.transport = state;
                self.status = match (self.engine.pause(), self.send_mmc(0x09)) {
                    (Ok(()), Ok(())) => "Paused; sent MMC Pause to DAW.".into(),
                    (Err(error), _) => format!("Could not pause CPAL input: {error}"),
                    (_, Err(error)) => format!("Paused locally; DAW MMC failed: {error}"),
                };
            }
            Transport::Playing => {
                let resume_error = (self.transport == Transport::Paused)
                    .then(|| self.engine.resume())
                    .transpose()
                    .err();
                if self.started_at.is_none() {
                    self.started_at = Some(Instant::now());
                }
                self.transport = state;
                self.status = match (resume_error, self.send_mmc(0x02)) {
                    (None, Ok(())) => "Playing; sent MMC Play to DAW.".into(),
                    (Some(error), _) => format!("Could not resume CPAL input: {error}"),
                    (_, Err(error)) => format!("Playing locally; DAW MMC failed: {error}"),
                };
            }
        }
    }
}
impl eframe::App for DawApp {
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        ctx.request_repaint_after(Duration::from_millis(33));
        let bg = Color32::from_rgb(10, 16, 22);
        egui::CentralPanel::default()
            .frame(egui::Frame::default().fill(bg).inner_margin(8.0))
            .show(ctx, |ui| {
                transport_bar(ui, self);
                ui.add_space(6.0);
                ui.columns(2, |columns| {
                    spatial_panner(&mut columns[0], self);
                    io_panel(&mut columns[1], self);
                });
                ui.add_space(6.0);
                timeline(ui, self.elapsed(), &mut self.track_armed);
                ui.add_space(6.0);
                ui.columns(2, |columns| {
                    automation(&mut columns[0]);
                    mixer(
                        &mut columns[1],
                        &mut self.subgroup_levels,
                        &mut self.master_level,
                    );
                });
                ui.add_space(4.0);
                ui.colored_label(
                    Color32::from_rgb(128, 185, 205),
                    format!("ENGINE: CPAL • {}", self.status),
                );
            });
    }
}

fn panel(ui: &mut egui::Ui, title: &str, add: impl FnOnce(&mut egui::Ui)) {
    egui::Frame::group(ui.style())
        .fill(Color32::from_rgb(17, 29, 38))
        .show(ui, |ui| {
            ui.colored_label(Color32::from_rgb(96, 210, 240), title);
            ui.separator();
            add(ui);
        });
}
fn transport_bar(ui: &mut egui::Ui, app: &mut DawApp) {
    egui::Frame::none()
        .fill(Color32::from_rgb(5, 10, 15))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.heading(
                    egui::RichText::new("AUDIO CONSOLE").color(Color32::from_rgb(96, 210, 240)),
                );
                ui.label("17 CHANNEL • 9.2.6");
                ui.separator();
                let record = ui
                    .add(egui::Button::new("● REC").fill(Color32::from_rgb(130, 28, 38)))
                    .clicked();
                let play = ui.button("▶ PLAY").clicked();
                let pause = ui.button("Ⅱ PAUSE").clicked();
                let stop = ui.button("■ STOP").clicked();
                if record {
                    app.set_transport(Transport::Recording);
                }
                if play {
                    app.set_transport(Transport::Playing);
                }
                if pause {
                    app.set_transport(Transport::Paused);
                }
                if stop {
                    app.set_transport(Transport::Stopped);
                }
                ui.separator();
                let t = app.elapsed();
                ui.monospace(format!(
                    "{:02}:{:02}:{:02}:{:02}",
                    t.as_secs() / 3600,
                    (t.as_secs() / 60) % 60,
                    t.as_secs() % 60,
                    (t.subsec_millis() * 24) / 1000
                ));
                ui.colored_label(
                    if app.transport == Transport::Recording {
                        Color32::RED
                    } else {
                        Color32::LIGHT_GRAY
                    },
                    app.transport.label(),
                );
            })
        });
}
fn spatial_panner(ui: &mut egui::Ui, app: &mut DawApp) {
    panel(ui, "SPATIAL PANNER • 9.2.6 • 17 SPEAKERS", |ui| {
        let (rect, _) =
            ui.allocate_exact_size(Vec2::new(ui.available_width(), 230.0), Sense::hover());
        let painter = ui.painter_at(rect);
        painter.rect_filled(rect, 4.0, Color32::from_rgb(8, 18, 25));
        vu_meter(
            &painter,
            Rect::from_min_size(rect.min + Vec2::new(8.0, 28.0), Vec2::new(18.0, 170.0)),
            app.engine.peak(0).max(0.45),
            "L",
        );
        vu_meter(
            &painter,
            Rect::from_min_size(
                Pos2::new(rect.max.x - 26.0, rect.min.y + 28.0),
                Vec2::new(18.0, 170.0),
            ),
            app.engine.peak(1).max(0.35),
            "R",
        );
        let center = rect.center() + Vec2::new(0.0, 8.0);
        let radius = rect.width().min(310.0) * 0.32;
        painter.circle_stroke(center, radius, Stroke::new(1.0_f32, Color32::from_gray(65)));
        for (index, name) in CHANNEL_LAYOUT.iter().enumerate() {
            let (x, y) = speaker_position(index);
            let pos = center + Vec2::new(x * radius, y * radius);
            let response = ui.interact(
                Rect::from_center_size(pos, Vec2::splat(25.0)),
                ui.id().with(index),
                Sense::click(),
            );
            if response.clicked() {
                app.selected_speaker = index;
            }
            let color = if index == app.selected_speaker {
                Color32::from_rgb(250, 182, 55)
            } else if index >= 11 {
                Color32::from_rgb(218, 112, 218)
            } else if index >= 9 {
                Color32::from_rgb(240, 180, 55)
            } else {
                Color32::from_rgb(84, 196, 228)
            };
            painter.circle_filled(pos, 9.0, color);
            painter.text(
                pos + Vec2::new(0.0, 13.0),
                egui::Align2::CENTER_TOP,
                *name,
                egui::FontId::monospace(10.0),
                Color32::WHITE,
            );
        }
        painter.text(
            center,
            egui::Align2::CENTER_CENTER,
            "OBJECT",
            egui::FontId::proportional(11.0),
            Color32::WHITE,
        );
        ui.label(format!(
            "Selected speaker: {}  •  click a node to select",
            CHANNEL_LAYOUT[app.selected_speaker]
        ));
    });
}
fn speaker_position(i: usize) -> (f32, f32) {
    const POS: [(f32, f32); 17] = [
        (-0.70, -0.68),
        (0.70, -0.68),
        (0.0, -0.82),
        (-1.0, -0.30),
        (1.0, -0.30),
        (-1.0, 0.35),
        (1.0, 0.35),
        (-0.68, 0.85),
        (0.68, 0.85),
        (-0.23, -0.15),
        (0.23, -0.15),
        (-0.62, -1.15),
        (0.62, -1.15),
        (-0.42, -0.30),
        (0.42, -0.30),
        (-0.62, 0.42),
        (0.62, 0.42),
    ];
    POS[i]
}
fn vu_meter(p: &egui::Painter, rect: Rect, level: f32, label: &str) {
    p.rect_filled(rect, 2.0, Color32::from_rgb(4, 7, 9));
    let lit = (level.clamp(0.0, 1.0) * 14.0).ceil() as usize;
    for i in 0..14 {
        let y = rect.max.y - (i as f32 + 1.0) * rect.height() / 14.0;
        let c = if i >= 11 {
            Color32::RED
        } else if i >= 8 {
            Color32::YELLOW
        } else {
            Color32::from_rgb(55, 220, 125)
        };
        p.rect_filled(
            Rect::from_min_size(
                Pos2::new(rect.min.x + 3.0, y + 1.0),
                Vec2::new(rect.width() - 6.0, rect.height() / 14.0 - 2.0),
            ),
            1.0,
            if i < lit { c } else { Color32::from_gray(35) },
        );
    }
    p.text(
        Pos2::new(rect.center().x, rect.max.y + 5.0),
        egui::Align2::CENTER_TOP,
        label,
        egui::FontId::monospace(11.0),
        Color32::WHITE,
    );
}
fn io_panel(ui: &mut egui::Ui, app: &mut DawApp) {
    panel(ui, "AUDIO I/O • CPAL + DAW MIDI", |ui| {
        ui.label("Input device");
        egui::ComboBox::from_id_salt("input")
            .selected_text(
                app.input_names
                    .get(app.selected_input)
                    .map(String::as_str)
                    .unwrap_or("No input available"),
            )
            .show_ui(ui, |ui| {
                for (i, name) in app.input_names.iter().enumerate() {
                    ui.selectable_value(&mut app.selected_input, i, name);
                }
            });
        ui.label("Output device");
        egui::ComboBox::from_id_salt("output")
            .selected_text(
                app.output_names
                    .get(app.selected_output)
                    .map(String::as_str)
                    .unwrap_or("No output available"),
            )
            .show_ui(ui, |ui| {
                for (i, name) in app.output_names.iter().enumerate() {
                    ui.selectable_value(&mut app.selected_output, i, name);
                }
            });
        ui.label("DAW MMC MIDI output");
        egui::ComboBox::from_id_salt("midi-output")
            .selected_text(
                app.midi_port_names
                    .get(app.selected_midi_port)
                    .map(String::as_str)
                    .unwrap_or("No MIDI output available"),
            )
            .show_ui(ui, |ui| {
                for (i, name) in app.midi_port_names.iter().enumerate() {
                    if ui
                        .selectable_value(&mut app.selected_midi_port, i, name)
                        .changed()
                    {
                        app.midi_out = None;
                    }
                }
            });
        ui.separator();
        ui.label("Capture uses the selected CPAL device’s default stream.");
        ui.label("One 32-bit float WAV is created for every input channel.");
        ui.label("REC sends MIDI Machine Control Record Strobe to this port.");
    });
}
fn timeline(ui: &mut egui::Ui, elapsed: Duration, armed: &mut [bool; 8]) {
    panel(ui, "TIMECODE GRID • TIME BAR • TRACK VIEW", |ui| {
        let width = ui.available_width();
        let (ruler, _) = ui.allocate_exact_size(Vec2::new(width, 28.0), Sense::hover());
        let p = ui.painter_at(ruler);
        for second in 0..=12 {
            let x = ruler.left() + second as f32 * ruler.width() / 12.0;
            p.line_segment(
                [
                    Pos2::new(x, ruler.top() + 12.0),
                    Pos2::new(x, ruler.bottom()),
                ],
                Stroke::new(1.0_f32, Color32::from_gray(90)),
            );
            p.text(
                Pos2::new(x + 2.0, ruler.top()),
                egui::Align2::LEFT_TOP,
                format!("01:00:{second:02}:00"),
                egui::FontId::monospace(9.0),
                Color32::LIGHT_GRAY,
            );
        }
        let playhead = ruler.left() + (elapsed.as_secs_f32() % 12.0) / 12.0 * ruler.width();
        p.line_segment(
            [
                Pos2::new(playhead, ruler.top()),
                Pos2::new(playhead, ruler.bottom() + 178.0),
            ],
            Stroke::new(2.0_f32, Color32::from_rgb(245, 78, 68)),
        );
        egui::ScrollArea::vertical()
            .max_height(170.0)
            .show(ui, |ui| {
                for (i, name) in TRACK_NAMES.iter().enumerate() {
                    ui.horizontal(|ui| {
                        ui.checkbox(&mut armed[i], "");
                        ui.monospace(format!("{:02}", i + 1));
                        ui.label(*name);
                        let (clip, _) = ui.allocate_exact_size(
                            Vec2::new(ui.available_width(), 22.0),
                            Sense::hover(),
                        );
                        let p = ui.painter_at(clip);
                        p.rect_filled(
                            clip,
                            2.0,
                            if armed[i] {
                                Color32::from_rgb(62, 36, 45)
                            } else {
                                Color32::from_rgb(26, 52, 65)
                            },
                        );
                        for x in (0..clip.width() as i32).step_by(8) {
                            let h = 3.0 + ((x / 8 + i as i32 * 3) % 12) as f32;
                            p.line_segment(
                                [
                                    Pos2::new(clip.left() + x as f32, clip.center().y - h),
                                    Pos2::new(clip.left() + x as f32, clip.center().y + h),
                                ],
                                Stroke::new(1.0_f32, Color32::from_rgb(102, 205, 230)),
                            );
                        }
                    });
                }
            });
    });
}
fn automation(ui: &mut egui::Ui) {
    panel(ui, "AUTOMATION", |ui| {
        for (name, color) in [
            ("Master volume", Color32::from_rgb(250, 170, 50)),
            ("SG 1 volume", Color32::from_rgb(96, 210, 240)),
            ("Vox pan", Color32::from_rgb(200, 120, 220)),
        ] {
            ui.horizontal(|ui| {
                ui.label(name);
                let (rect, _) =
                    ui.allocate_exact_size(Vec2::new(ui.available_width(), 26.0), Sense::hover());
                let p = ui.painter_at(rect);
                let middle = Pos2::new(rect.center().x, rect.top() + 5.0);
                p.line_segment([rect.left_center(), middle], Stroke::new(1.5_f32, color));
                p.line_segment([middle, rect.right_center()], Stroke::new(1.5_f32, color));
            });
        }
    });
}
fn mixer(ui: &mut egui::Ui, levels: &mut [f32; 8], master: &mut f32) {
    panel(ui, "MIXER • 8 SUBGROUPS + MASTER FADER", |ui| {
        ui.horizontal(|ui| {
            for (i, level) in levels.iter_mut().enumerate() {
                strip(
                    ui,
                    &format!("SG{}", i + 1),
                    level,
                    Color32::from_rgb(85, 194, 225),
                );
            }
            ui.separator();
            strip(ui, "MASTER", master, Color32::from_rgb(250, 170, 50));
        });
    });
}
fn strip(ui: &mut egui::Ui, name: &str, level: &mut f32, color: Color32) {
    ui.vertical(|ui| {
        ui.colored_label(color, name);
        ui.add(
            egui::Slider::new(level, 0.0..=1.0)
                .vertical()
                .show_value(false),
        );
        ui.monospace(format!("{:.1} dB", *level * 60.0 - 60.0));
    });
}
fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1180.0, 820.0])
            .with_min_inner_size([900.0, 650.0])
            .with_title("Audio Console — 9.2.6"),
        ..Default::default()
    };
    eframe::run_native(
        "Audio Console",
        options,
        Box::new(|_| Ok(Box::new(DawApp::default()))),
    )
}
