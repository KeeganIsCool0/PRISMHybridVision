use eframe::egui;

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([400.0, 500.0])
            .with_title("PRISM Audio Calculator"),
        ..Default::default()
    };
    eframe::run_native(
        "PRISM Audio Calculator",
        options,
        Box::new(|cc| Ok(Box::new(AudioCalculatorApp::new(cc)))),
    )
}

struct AudioCalculatorApp {
    input_voltage: String,
    gain: String,
    impedance: String,
    eq_frequency: String,
    compressor_threshold: String,
    attack: String,
    release: String,
    output_voltage: String,
    output_power: String,
    output_current: String,
}

impl AudioCalculatorApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            input_voltage: String::new(),
            gain: String::new(),
            impedance: String::new(),
            eq_frequency: String::new(),
            compressor_threshold: String::new(),
            attack: String::new(),
            release: String::new(),
            output_voltage: String::new(),
            output_power: String::new(),
            output_current: String::new(),
        }
    }
}

impl eframe::App for AudioCalculatorApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("PRISM Audio Calculator");
            ui.separator();

            egui::Grid::new("inputs")
                .num_columns(2)
                .spacing([10.0, 10.0])
                .show(ui, |ui| {
                    ui.label("Input Voltage (V):");
                    ui.text_edit_singleline(&mut self.input_voltage);
                    ui.end_row();

                    ui.label("Gain (dB):");
                    ui.text_edit_singleline(&mut self.gain);
                    ui.end_row();

                    ui.label("Impedance (Ω):");
                    ui.text_edit_singleline(&mut self.impedance);
                    ui.end_row();

                    ui.label("EQ Frequency (Hz):");
                    ui.text_edit_singleline(&mut self.eq_frequency);
                    ui.end_row();

                    ui.label("Comp Threshold (dB):");
                    ui.text_edit_singleline(&mut self.compressor_threshold);
                    ui.end_row();

                    ui.label("Attack (ms):");
                    ui.text_edit_singleline(&mut self.attack);
                    ui.end_row();

                    ui.label("Release (ms):");
                    ui.text_edit_singleline(&mut self.release);
                    ui.end_row();
                });

            ui.separator();
            ui.heading("Calculations");

            if ui.button("Calculate Output Voltage").clicked() {
                self.calculate_output_voltage();
            }
            if ui.button("Calculate Output Power").clicked() {
                self.calculate_output_power();
            }
            if ui.button("Calculate Output Current").clicked() {
                self.calculate_output_current();
            }
            if ui.button("Convert Gain to Ratio").clicked() {
                self.convert_gain_to_ratio();
            }
            if ui.button("Convert dB to Voltage Ratio").clicked() {
                self.convert_db_to_voltage_ratio();
            }

            ui.separator();
            ui.label(format!("Output Voltage: {} V", self.output_voltage));
            ui.label(format!("Output Power: {} W", self.output_power));
            ui.label(format!("Output Current: {} A", self.output_current));
        });
    }
}

impl AudioCalculatorApp {
    fn calculate_output_voltage(&mut self) {
        let vin = self.input_voltage.trim().parse::<f64>().unwrap_or(0.0);
        let gain_db = self.gain.trim().parse::<f64>().unwrap_or(0.0);
        // Vout = Vin * 10^(Gain/20)
        let vout = vin * 10f64.powf(gain_db / 20.0);
        self.output_voltage = format!("{:.3}", vout);
    }

    fn calculate_output_power(&mut self) {
        let vout = self.output_voltage.trim().parse::<f64>().unwrap_or(0.0);
        let impedance = self.impedance.trim().parse::<f64>().unwrap_or(1.0);
        // P = V^2 / R
        if impedance > 0.0 {
            let power = vout.powi(2) / impedance;
            self.output_power = format!("{:.3}", power);
        } else {
            self.output_power = "∞ (zero impedance)".to_string();
        }
    }

    fn calculate_output_current(&mut self) {
        let vout = self.output_voltage.trim().parse::<f64>().unwrap_or(0.0);
        let impedance = self.impedance.trim().parse::<f64>().unwrap_or(1.0);
        // I = V / R
        if impedance > 0.0 {
            let current = vout / impedance;
            self.output_current = format!("{:.3}", current);
        } else {
            self.output_current = "∞ (zero impedance)".to_string();
        }
    }

    fn convert_gain_to_ratio(&mut self) {
        let gain_db = self.gain.trim().parse::<f64>().unwrap_or(0.0);
        // Ratio = 10^(Gain/20)
        let ratio = 10f64.powf(gain_db / 20.0);
        self.output_voltage = format!("{:.3} (ratio)", ratio);
    }

    fn convert_db_to_voltage_ratio(&mut self) {
        let db = self.compressor_threshold.trim().parse::<f64>().unwrap_or(0.0);
        // Voltage ratio = 10^(dB/20)
        let ratio = 10f64.powf(db / 20.0);
        self.output_power = format!("{:.3} (voltage ratio)", ratio);
    }
}