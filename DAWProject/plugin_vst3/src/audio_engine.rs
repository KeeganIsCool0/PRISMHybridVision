/// Audio engine that handles mixing and processing
pub struct AudioEngine {
    // Mixing state
    master_volume: f32,         // 0.0 to 2.0 (linear)
    master_pan: f32,            // -1.0 to 1.0
    subgroup_volumes: [f32; 8], // 0.0 to 2.0 for each subgroup
    // Input gains
    input_gains: Vec<f32>,
}

impl AudioEngine {
    pub fn new(max_input_channels: usize, _max_output_channels: usize) -> Self {
        Self {
            master_volume: 1.0,
            master_pan: 0.0,
            subgroup_volumes: [1.0; 8],
            input_gains: vec![1.0; max_input_channels],
        }
    }

    pub fn set_parameter(&mut self, id: u32, value: f32) {
        // Update audio processing parameters
        match id {
            1000 => self.master_volume = value, // Master Volume
            1001 => self.master_pan = value,    // Master Pan
            1002 => {
                // Input Gain (assuming first input)
                if !self.input_gains.is_empty() {
                    self.input_gains[0] = value;
                }
            }
            1010..=1017 => {
                // Subgroup volumes
                let index = (id - 1010) as usize;
                if index < self.subgroup_volumes.len() {
                    self.subgroup_volumes[index] = value;
                }
            }
            _ => {}
        }
    }

    /// Process channel-major buffers. The VST wrapper supplies host buffers;
    /// CPAL device I/O belongs to the standalone console, not the plug-in.
    pub fn process_audio(&mut self, input: &[Vec<f32>], output: &mut [Vec<f32>]) {
        let left_gain = (1.0 - self.master_pan).clamp(0.0, 1.0) * self.master_volume;
        let right_gain = (1.0 + self.master_pan).clamp(0.0, 1.0) * self.master_volume;
        for (channel, (in_ch, out_ch)) in input.iter().zip(output.iter_mut()).enumerate() {
            let input_gain = self.input_gains.get(channel).copied().unwrap_or(1.0);
            let pan_gain = if channel == 0 {
                left_gain
            } else if channel == 1 {
                right_gain
            } else {
                self.master_volume
            };
            for (source, destination) in in_ch.iter().zip(out_ch.iter_mut()) {
                *destination = source * input_gain * pan_gain;
            }
        }
    }
}

impl Default for AudioEngine {
    fn default() -> Self {
        Self::new(2, 2) // Default stereo
    }
}
