// Import ParamID and ParamValue from the correct location in vst3
use vst3::{Steinberg::Vst::ParamID, Steinberg::Vst::ParamValue};

/// Describes a plugin parameter
#[derive(Clone, Copy)]
pub struct ParameterDescriptor {
    pub id: ParamID,
    pub name: &'static str,
    pub short_name: &'static str,
    pub units: &'static str,
    pub min_value: ParamValue,
    pub max_value: ParamValue,
    pub default_value: ParamValue,
    pub is_stepped: bool,
    pub parameter_type: ParameterType,
}

impl ParameterDescriptor {
    pub fn value_to_string(&self, value: ParamValue) -> String {
        match self.parameter_type {
            ParameterType::Boolean => if value > 0.5 { "On" } else { "Off" }.to_string(),
            ParameterType::Integer => {
                let int_val =
                    (value * (self.max_value - self.min_value) + self.min_value).round() as i32;
                int_val.to_string()
            }
            ParameterType::Number => {
                if self.units == "dB" {
                    format!("{:.1} dB", value)
                } else if self.units == "Hz" {
                    format!("{:.0} Hz", value)
                } else if self.units == "ms" {
                    format!("{:.1} ms", value)
                } else if self.units == "%" {
                    format!("{:.0} %", value * 100.0)
                } else {
                    format!("{:.3}", value)
                }
            }
            ParameterType::Enum => {
                // For simplicity, just show the value
                format!(
                    "{:.0}",
                    value * (self.max_value - self.min_value) + self.min_value
                )
            }
        }
    }

    pub fn string_to_value(&self, s: &str) -> Option<ParamValue> {
        // Simple implementation - in reality this would be more robust
        match self.parameter_type {
            ParameterType::Boolean => match s.to_lowercase().as_str() {
                "on" | "true" | "1" => Some(1.0),
                "off" | "false" | "0" => Some(0.0),
                _ => None,
            },
            ParameterType::Integer => s.parse::<i32>().ok().map(|v| {
                ((v as f64 - self.min_value) / (self.max_value - self.min_value)).clamp(0.0, 1.0)
                    as ParamValue
            }),
            ParameterType::Number => {
                // Try to parse as number, handling units
                let replaced = s.replace(self.units, "");
                let s_clean = replaced.trim();
                if let Ok(v) = s_clean.parse::<f64>() {
                    Some(
                        ((v - self.min_value as f64) / (self.max_value - self.min_value) as f64)
                            .clamp(0.0, 1.0),
                    )
                } else {
                    None
                }
            }
            ParameterType::Enum => {
                // Try to parse as integer, handling units
                let replaced = s.replace(self.units, "");
                let s_clean = replaced.trim();
                if let Ok(v) = s_clean.parse::<i32>() {
                    Some(
                        ((v as f64 - self.min_value) / (self.max_value - self.min_value))
                            .clamp(0.0, 1.0),
                    )
                } else {
                    None
                }
            }
        }
    }

    pub fn set_value_normalized(&mut self, _value: ParamValue) {
        // This method doesn't make sense on the descriptor itself
        // The actual parameter value would be stored elsewhere
    }

    pub fn value_normalized(&self) -> ParamValue {
        // Return default value normalized
        ((self.default_value - self.min_value) / (self.max_value - self.min_value)).clamp(0.0, 1.0)
    }
}

/// Parameter types matching VST3
#[derive(Clone, Copy)]
pub enum ParameterType {
    Boolean,
    Integer,
    Number,
    Enum,
}

/// Define our plugin parameters
pub struct Parameters;

impl Parameters {
    pub const MASTER_VOLUME: ParamID = 1000;
    pub const MASTER_PAN: ParamID = 1001;
    pub const INPUT_GAIN: ParamID = 1002;
    pub const SUB_GROUP_1_VOLUME: ParamID = 1010;
    pub const SUB_GROUP_2_VOLUME: ParamID = 1011;
    pub const SUB_GROUP_3_VOLUME: ParamID = 1012;
    pub const SUB_GROUP_4_VOLUME: ParamID = 1013;
    pub const SUB_GROUP_5_VOLUME: ParamID = 1014;
    pub const SUB_GROUP_6_VOLUME: ParamID = 1015;
    pub const SUB_GROUP_7_VOLUME: ParamID = 1016;
    pub const SUB_GROUP_8_VOLUME: ParamID = 1017;
    pub const RECORD_ARM_TRACK_1: ParamID = 1100;
    pub const RECORD_ARM_TRACK_2: ParamID = 1101;
    pub const RECORD_ARM_TRACK_3: ParamID = 1102;
    pub const RECORD_ARM_TRACK_4: ParamID = 1103;
    pub const PLAYBACK_STATE: ParamID = 1200; // 0=stop, 1=play, 2=record

    pub const PARAMETERS: &'static [ParameterDescriptor] = &[
        // Sub-group volumes
        ParameterDescriptor {
            id: Self::SUB_GROUP_1_VOLUME,
            name: "Sub Group 1 Volume",
            short_name: "SG1 Vol",
            units: "dB",
            min_value: -60.0,
            max_value: 6.0,
            default_value: 0.0,
            is_stepped: false,
            parameter_type: ParameterType::Number,
        },
        ParameterDescriptor {
            id: Self::SUB_GROUP_2_VOLUME,
            name: "Sub Group 2 Volume",
            short_name: "SG2 Vol",
            units: "dB",
            min_value: -60.0,
            max_value: 6.0,
            default_value: 0.0,
            is_stepped: false,
            parameter_type: ParameterType::Number,
        },
        ParameterDescriptor {
            id: Self::SUB_GROUP_3_VOLUME,
            name: "Sub Group 3 Volume",
            short_name: "SG3 Vol",
            units: "dB",
            min_value: -60.0,
            max_value: 6.0,
            default_value: 0.0,
            is_stepped: false,
            parameter_type: ParameterType::Number,
        },
        ParameterDescriptor {
            id: Self::SUB_GROUP_4_VOLUME,
            name: "Sub Group 4 Volume",
            short_name: "SG4 Vol",
            units: "dB",
            min_value: -60.0,
            max_value: 6.0,
            default_value: 0.0,
            is_stepped: false,
            parameter_type: ParameterType::Number,
        },
        ParameterDescriptor {
            id: Self::SUB_GROUP_5_VOLUME,
            name: "Sub Group 5 Volume",
            short_name: "SG5 Vol",
            units: "dB",
            min_value: -60.0,
            max_value: 6.0,
            default_value: 0.0,
            is_stepped: false,
            parameter_type: ParameterType::Number,
        },
        ParameterDescriptor {
            id: Self::SUB_GROUP_6_VOLUME,
            name: "Sub Group 6 Volume",
            short_name: "SG6 Vol",
            units: "dB",
            min_value: -60.0,
            max_value: 6.0,
            default_value: 0.0,
            is_stepped: false,
            parameter_type: ParameterType::Number,
        },
        ParameterDescriptor {
            id: Self::SUB_GROUP_7_VOLUME,
            name: "Sub Group 7 Volume",
            short_name: "SG7 Vol",
            units: "dB",
            min_value: -60.0,
            max_value: 6.0,
            default_value: 0.0,
            is_stepped: false,
            parameter_type: ParameterType::Number,
        },
        ParameterDescriptor {
            id: Self::SUB_GROUP_8_VOLUME,
            name: "Sub Group 8 Volume",
            short_name: "SG8 Vol",
            units: "dB",
            min_value: -60.0,
            max_value: 6.0,
            default_value: 0.0,
            is_stepped: false,
            parameter_type: ParameterType::Number,
        },
        // Record Arm section
        ParameterDescriptor {
            id: Self::RECORD_ARM_TRACK_1,
            name: "Record Arm Track 1",
            short_name: "Rec Arm 1",
            units: "",
            min_value: 0.0,
            max_value: 1.0,
            default_value: 0.0,
            is_stepped: true,
            parameter_type: ParameterType::Boolean,
        },
        ParameterDescriptor {
            id: Self::RECORD_ARM_TRACK_2,
            name: "Record Arm Track 2",
            short_name: "Rec Arm 2",
            units: "",
            min_value: 0.0,
            max_value: 1.0,
            default_value: 0.0,
            is_stepped: true,
            parameter_type: ParameterType::Boolean,
        },
        ParameterDescriptor {
            id: Self::RECORD_ARM_TRACK_3,
            name: "Record Arm Track 3",
            short_name: "Rec Arm 3",
            units: "",
            min_value: 0.0,
            max_value: 1.0,
            default_value: 0.0,
            is_stepped: true,
            parameter_type: ParameterType::Boolean,
        },
        ParameterDescriptor {
            id: Self::RECORD_ARM_TRACK_4,
            name: "Record Arm Track 4",
            short_name: "Rec Arm 4",
            units: "",
            min_value: 0.0,
            max_value: 1.0,
            default_value: 0.0,
            is_stepped: true,
            parameter_type: ParameterType::Boolean,
        },
        // Transport section
        ParameterDescriptor {
            id: Self::PLAYBACK_STATE,
            name: "Playback State",
            short_name: "State",
            units: "",
            min_value: 0.0,
            max_value: 2.0,
            default_value: 0.0,
            is_stepped: true,
            parameter_type: ParameterType::Enum,
        },
    ];
}
