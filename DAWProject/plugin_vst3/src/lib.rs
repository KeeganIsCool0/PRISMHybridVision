#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use std::ffi::{c_char, c_void, CString};
use std::ptr;
use std::slice;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use vst3::{uid, Class, ComWrapper, Steinberg::Vst::*, Steinberg::*};

mod audio_engine;
mod parameters;

// Helper functions from the gain.rs example
fn copy_cstring(src: &str, dst: &mut [c_char]) {
    let c_string = CString::new(src).unwrap_or_else(|_| CString::default());
    let bytes = c_string.as_bytes_with_nul();

    for (src, dst) in bytes.iter().zip(dst.iter_mut()) {
        *dst = *src as c_char;
    }

    if bytes.len() > dst.len() {
        if let Some(last) = dst.last_mut() {
            *last = 0;
        }
    }
}

fn copy_wstring(src: &str, dst: &mut [u16]) {
    let mut len = 0;
    for (src, dst) in src.encode_utf16().zip(dst.iter_mut()) {
        *dst = src as u16;
        len += 1;
    }

    if len < dst.len() {
        dst[len] = 0;
    } else if let Some(last) = dst.last_mut() {
        *last = 0;
    }
}

unsafe fn len_wstring(string: *const u16) -> usize {
    let mut len = 0;

    while *string.offset(len as isize) != 0 {
        len += 1;
    }

    len
}

// Unique IDs for our plugin components
const PROCESSOR_CID: TUID = uid(0x6E332252, 0x54224A00, 0xAA69301A, 0xF318797E);
const CONTROLLER_CID: TUID = uid(0x1BA8A477, 0xEE0A4A2D, 0x80F50D14, 0x13D2EAA1);

// Plugin name
const PLUGIN_NAME: &'static str = "Audio Console Plugin";

// Audio engine instance shared between processor and controller
static AUDIO_ENGINE: once_cell::sync::Lazy<Arc<Mutex<audio_engine::AudioEngine>>> =
    once_cell::sync::Lazy::new(|| Arc::new(Mutex::new(audio_engine::AudioEngine::new(2, 2))));

struct AudioConsoleProcessor {
    // We'll store parameter values here for audio processing
    // These are mirrored from the audio engine for quick access
    master_volume: AtomicU64, // stored as f64 bits
    master_pan: AtomicU64,
    input_gain: AtomicU64,
    subgroup_volumes: [AtomicU64; 8],
}

impl AudioConsoleProcessor {
    fn new() -> Self {
        // Initialize parameters to default values
        Self {
            master_volume: AtomicU64::new(1.0f64.to_bits()), // 1.0 (unity gain)
            master_pan: AtomicU64::new(0.0f64.to_bits()),    // center
            input_gain: AtomicU64::new(1.0f64.to_bits()),    // unity gain
            subgroup_volumes: [
                AtomicU64::new(1.0f64.to_bits()),
                AtomicU64::new(1.0f64.to_bits()),
                AtomicU64::new(1.0f64.to_bits()),
                AtomicU64::new(1.0f64.to_bits()),
                AtomicU64::new(1.0f64.to_bits()),
                AtomicU64::new(1.0f64.to_bits()),
                AtomicU64::new(1.0f64.to_bits()),
                AtomicU64::new(1.0f64.to_bits()),
            ],
        }
    }
}

impl Class for AudioConsoleProcessor {
    type Interfaces = (IComponent, IAudioProcessor);
}

impl IPluginBaseTrait for AudioConsoleProcessor {
    unsafe fn initialize(&self, _context: *mut FUnknown) -> tresult {
        kResultOk
    }

    unsafe fn terminate(&self) -> tresult {
        kResultOk
    }
}

impl IComponentTrait for AudioConsoleProcessor {
    unsafe fn getControllerClassId(&self, class_id: *mut TUID) -> tresult {
        *class_id = CONTROLLER_CID;
        kResultOk
    }

    unsafe fn setIoMode(&self, _mode: IoMode) -> tresult {
        kResultOk
    }

    unsafe fn getBusCount(&self, mediaType: MediaType, dir: BusDirection) -> i32 {
        match mediaType as MediaTypes {
            MediaTypes_::kAudio => match dir as BusDirections {
                BusDirections_::kInput => 1,  // One stereo input bus
                BusDirections_::kOutput => 1, // One stereo output bus
                _ => 0,
            },
            _ => 0,
        }
    }

    unsafe fn getBusInfo(
        &self,
        mediaType: MediaType,
        dir: BusDirection,
        index: i32,
        bus: *mut BusInfo,
    ) -> tresult {
        match mediaType as MediaTypes {
            MediaTypes_::kAudio => match dir as BusDirections {
                BusDirections_::kInput => match index {
                    0 => {
                        let bus = &mut *bus;
                        bus.mediaType = MediaTypes_::kAudio as MediaType;
                        bus.direction = BusDirections_::kInput as BusDirection;
                        bus.channelCount = 2;
                        copy_wstring("Stereo Input", &mut bus.name);
                        bus.busType = BusTypes_::kMain as BusType;
                        bus.flags = BusInfo_::BusFlags_::kDefaultActive as u32;
                        kResultOk
                    }
                    _ => kInvalidArgument,
                },
                BusDirections_::kOutput => match index {
                    0 => {
                        let bus = &mut *bus;
                        bus.mediaType = MediaTypes_::kAudio as MediaType;
                        bus.direction = BusDirections_::kOutput as BusDirection;
                        bus.channelCount = 2;
                        copy_wstring("Stereo Output", &mut bus.name);
                        bus.busType = BusTypes_::kMain as BusType;
                        bus.flags = BusInfo_::BusFlags_::kDefaultActive as u32;
                        kResultOk
                    }
                    _ => kInvalidArgument,
                },
                _ => kInvalidArgument,
            },
            _ => kInvalidArgument,
        }
    }

    unsafe fn getRoutingInfo(
        &self,
        _in_info: *mut RoutingInfo,
        _out_info: *mut RoutingInfo,
    ) -> tresult {
        kNotImplemented
    }

    unsafe fn activateBus(
        &self,
        _media_type: MediaType,
        _dir: BusDirection,
        _index: i32,
        _state: TBool,
    ) -> tresult {
        kResultOk
    }

    unsafe fn setActive(&self, _state: TBool) -> tresult {
        kResultOk
    }

    unsafe fn setState(&self, _state: *mut IBStream) -> tresult {
        kResultOk
    }

    unsafe fn getState(&self, _state: *mut IBStream) -> tresult {
        kResultOk
    }
}

impl IAudioProcessorTrait for AudioConsoleProcessor {
    unsafe fn setBusArrangements(
        &self,
        inputs: *mut SpeakerArrangement,
        num_ins: i32,
        outputs: *mut SpeakerArrangement,
        num_outs: i32,
    ) -> tresult {
        if num_ins != 1 || num_outs != 1 {
            return kResultFalse;
        }

        if *inputs != SpeakerArr::kStereo || *outputs != SpeakerArr::kStereo {
            return kResultFalse;
        }

        kResultTrue
    }

    unsafe fn getBusArrangement(
        &self,
        dir: BusDirection,
        index: i32,
        arr: *mut SpeakerArrangement,
    ) -> tresult {
        match dir as BusDirections {
            BusDirections_::kInput => {
                if index == 0 {
                    *arr = SpeakerArr::kStereo;
                    kResultOk
                } else {
                    kInvalidArgument
                }
            }
            BusDirections_::kOutput => {
                if index == 0 {
                    *arr = SpeakerArr::kStereo;
                    kResultOk
                } else {
                    kInvalidArgument
                }
            }
            _ => kInvalidArgument,
        }
    }

    unsafe fn canProcessSampleSize(&self, _symbolic_sample_size: i32) -> tresult {
        kResultOk
    }

    unsafe fn getLatencySamples(&self) -> u32 {
        0
    }

    unsafe fn setupProcessing(&self, _setup: *mut ProcessSetup) -> tresult {
        kResultOk
    }

    unsafe fn setProcessing(&self, _state: TBool) -> tresult {
        kResultOk
    }

    unsafe fn process(&self, data: *mut ProcessData) -> tresult {
        let process_data = &*data;

        // Update audio engine with current parameter values
        if let Ok(mut engine) = AUDIO_ENGINE.lock() {
            // Update parameters from our atomic storage
            let master_volume = f64::from_bits(self.master_volume.load(Ordering::Relaxed)) as f32;
            let master_pan = f64::from_bits(self.master_pan.load(Ordering::Relaxed)) as f32;
            let input_gain = f64::from_bits(self.input_gain.load(Ordering::Relaxed)) as f32;
            let mut subgroup_volumes = [0.0f32; 8];
            for i in 0..8 {
                subgroup_volumes[i] =
                    f64::from_bits(self.subgroup_volumes[i].load(Ordering::Relaxed)) as f32;
            }

            engine.set_parameter(1000, master_volume); // Master Volume
            engine.set_parameter(1001, master_pan); // Master Pan
            engine.set_parameter(1002, input_gain); // Input Gain
            for i in 0..8 {
                engine.set_parameter(1010 + i as u32, subgroup_volumes[i]); // Subgroup volumes
            }

            // Process audio
            if process_data.numInputs == 1 && process_data.numOutputs == 1 {
                let input_buffers = unsafe {
                    slice::from_raw_parts(
                        (*process_data.inputs).__field0.channelBuffers32,
                        (*process_data.inputs).numChannels as usize,
                    )
                };
                let output_buffers = unsafe {
                    slice::from_raw_parts_mut(
                        (*process_data.outputs).__field0.channelBuffers32,
                        (*process_data.outputs).numChannels as usize,
                    )
                };

                if input_buffers.len() >= 2 && output_buffers.len() >= 2 {
                    let input_l =
                        slice::from_raw_parts(input_buffers[0], process_data.numSamples as usize);
                    let input_r =
                        slice::from_raw_parts(input_buffers[1], process_data.numSamples as usize);
                    let output_l = slice::from_raw_parts_mut(
                        output_buffers[0],
                        process_data.numSamples as usize,
                    );
                    let output_r = slice::from_raw_parts_mut(
                        output_buffers[1],
                        process_data.numSamples as usize,
                    );

                    // Convert to format expected by audio engine
                    let input_vec = vec![input_l.to_vec(), input_r.to_vec()];
                    let mut output_vec = vec![
                        vec![0.0; process_data.numSamples as usize],
                        vec![0.0; process_data.numSamples as usize],
                    ];

                    // Process audio through engine
                    engine.process_audio(&input_vec, &mut output_vec);

                    // Copy results back
                    output_l.copy_from_slice(&output_vec[0]);
                    output_r.copy_from_slice(&output_vec[1]);
                }
            }
        }

        kResultOk
    }

    unsafe fn getTailSamples(&self) -> u32 {
        0
    }
}

struct AudioConsoleController {
    // We don't need to store much state here since parameters are handled in the processor
    // But we could cache values for the UI if needed
}

impl AudioConsoleController {
    fn new() -> Self {
        Self {}
    }
}

impl Class for AudioConsoleController {
    type Interfaces = (IEditController,);
}

impl IPluginBaseTrait for AudioConsoleController {
    unsafe fn initialize(&self, _context: *mut FUnknown) -> tresult {
        kResultOk
    }

    unsafe fn terminate(&self) -> tresult {
        kResultOk
    }
}

impl IEditControllerTrait for AudioConsoleController {
    unsafe fn getParameterCount(&self) -> i32 {
        parameters::Parameters::PARAMETERS.len() as i32
    }

    unsafe fn getParameterInfo(&self, param_index: i32, info: *mut ParameterInfo) -> tresult {
        let param_index = param_index as usize;
        if param_index < parameters::Parameters::PARAMETERS.len() {
            let param = &parameters::Parameters::PARAMETERS[param_index];
            let info = &mut *info;

            info.id = param.id as ParamID;
            copy_wstring(param.name, &mut info.title);
            copy_wstring(param.short_name, &mut info.shortTitle);
            copy_wstring(param.units, &mut info.units);
            info.stepCount = if param.is_stepped { 1 } else { 0 };
            info.defaultNormalizedValue = ((param.default_value - param.min_value)
                / (param.max_value - param.min_value))
                .clamp(0.0, 1.0);
            info.unitId = 0;
            info.flags = ParameterInfo_::ParameterFlags_::kCanAutomate as i32;

            kResultOk
        } else {
            kInvalidArgument
        }
    }

    unsafe fn getParamStringByValue(
        &self,
        id: ParamID,
        value_normalized: ParamValue,
        string: *mut String128,
    ) -> tresult {
        let slice = unsafe { &mut *string };

        if let Some(param) = parameters::Parameters::PARAMETERS
            .iter()
            .find(|p| p.id == id)
        {
            let display = param.value_to_string(value_normalized);
            copy_wstring(&display, slice);
            kResultOk
        } else {
            kInvalidArgument
        }
    }

    unsafe fn getParamValueByString(
        &self,
        id: ParamID,
        string: *mut u16,
        value_normalized: *mut ParamValue,
    ) -> tresult {
        match id {
            _ => {
                if let Some(param) = parameters::Parameters::PARAMETERS
                    .iter()
                    .find(|p| p.id == id)
                {
                    let len = unsafe { len_wstring(string) };
                    if let Ok(string_val) =
                        String::from_utf16(slice::from_raw_parts(string as *const u16, len))
                    {
                        match param.string_to_value(&string_val) {
                            Some(value) => {
                                *value_normalized = value;
                                return kResultOk;
                            }
                            None => return kInvalidArgument,
                        }
                    } else {
                        return kInvalidArgument;
                    }
                } else {
                    return kInvalidArgument;
                }
            }
        }
    }

    unsafe fn normalizedParamToPlain(
        &self,
        id: ParamID,
        value_normalized: ParamValue,
    ) -> ParamValue {
        if let Some(param) = parameters::Parameters::PARAMETERS
            .iter()
            .find(|p| p.id == id)
        {
            param.min_value + value_normalized * (param.max_value - param.min_value)
        } else {
            0.0
        }
    }

    unsafe fn plainParamToNormalized(&self, id: ParamID, plain_value: ParamValue) -> ParamValue {
        if let Some(param) = parameters::Parameters::PARAMETERS
            .iter()
            .find(|p| p.id == id)
        {
            ((plain_value - param.min_value) / (param.max_value - param.min_value)).clamp(0.0, 1.0)
        } else {
            0.0
        }
    }

    unsafe fn getParamNormalized(&self, id: ParamID) -> ParamValue {
        // Get the current value from the processor
        if let Some(param) = parameters::Parameters::PARAMETERS
            .iter()
            .find(|p| p.id == id)
        {
            param.value_normalized()
        } else {
            0.0
        }
    }

    unsafe fn setParamNormalized(&self, id: ParamID, value: ParamValue) -> tresult {
        // Update the parameter in our audio engine/shared state
        if let Some(param) = parameters::Parameters::PARAMETERS
            .iter()
            .find(|p| p.id == id)
        {
            // Update the audio engine
            if let Ok(mut engine) = AUDIO_ENGINE.lock() {
                let actual_value = param.min_value + value * (param.max_value - param.min_value);
                engine.set_parameter(id, actual_value as f32);
            }
            kResultOk
        } else {
            kInvalidArgument
        }
    }

    unsafe fn setComponentHandler(&self, _handler: *mut IComponentHandler) -> tresult {
        kResultOk
    }

    unsafe fn setComponentState(&self, _state: *mut IBStream) -> tresult {
        kResultOk
    }

    unsafe fn setState(&self, _state: *mut IBStream) -> tresult {
        kResultOk
    }

    unsafe fn getState(&self, _state: *mut IBStream) -> tresult {
        kResultOk
    }

    unsafe fn createView(&self, _name: *const c_char) -> *mut IPlugView {
        // We're not providing a VST3 GUI, we're using our own separate GUI
        ptr::null_mut()
    }
}

struct Factory {}

impl Class for Factory {
    type Interfaces = (IPluginFactory,);
}

impl IPluginFactoryTrait for Factory {
    unsafe fn getFactoryInfo(&self, info: *mut PFactoryInfo) -> tresult {
        let info = &mut *info;

        copy_cstring("MyCo", &mut info.vendor);
        copy_cstring("https://myco.example.com", &mut info.url);
        copy_cstring("info@myco.example.com", &mut info.email);
        info.flags = PFactoryInfo_::FactoryFlags_::kUnicode as int32;

        kResultOk
    }

    unsafe fn countClasses(&self) -> i32 {
        2
    }

    unsafe fn getClassInfo(&self, index: i32, info: *mut PClassInfo) -> tresult {
        match index {
            0 => {
                let info = &mut *info;
                info.cid = PROCESSOR_CID;
                info.cardinality = PClassInfo_::ClassCardinality_::kManyInstances as int32;
                copy_cstring("Audio Processor Class", &mut info.category);
                copy_cstring(PLUGIN_NAME, &mut info.name);

                kResultOk
            }
            1 => {
                let info = &mut *info;
                info.cid = CONTROLLER_CID;
                info.cardinality = PClassInfo_::ClassCardinality_::kManyInstances as int32;
                copy_cstring("Audio Controller Class", &mut info.category);
                copy_cstring(PLUGIN_NAME, &mut info.name);

                kResultOk
            }
            _ => kInvalidArgument,
        }
    }

    unsafe fn createInstance(
        &self,
        cid: FIDString,
        iid: FIDString,
        obj: *mut *mut c_void,
    ) -> tresult {
        let instance = match *(cid as *const TUID) {
            PROCESSOR_CID => Some(
                ComWrapper::new(AudioConsoleProcessor::new())
                    .to_com_ptr::<FUnknown>()
                    .unwrap(),
            ),
            CONTROLLER_CID => Some(
                ComWrapper::new(AudioConsoleController::new())
                    .to_com_ptr::<FUnknown>()
                    .unwrap(),
            ),
            _ => None,
        };

        if let Some(instance) = instance {
            let ptr = instance.as_ptr();
            ((*(*ptr).vtbl).queryInterface)(ptr, iid as *mut TUID, obj)
        } else {
            kInvalidArgument
        }
    }
}

// Entry points for the VST3 plugin
#[cfg(target_os = "windows")]
#[no_mangle]
extern "system" fn InitDll() -> bool {
    true
}

#[cfg(target_os = "windows")]
#[no_mangle]
extern "system" fn ExitDll() -> bool {
    true
}

#[cfg(target_os = "macos")]
#[no_mangle]
extern "system" fn BundleEntry(_bundle_ref: *mut c_void) -> bool {
    true
}

#[cfg(target_os = "macos")]
#[no_mangle]
extern "system" fn BundleExit() -> bool {
    true
}

#[cfg(target_os = "linux")]
#[no_mangle]
extern "system" fn ModuleEntry(_library_handle: *mut c_void) -> bool {
    true
}

#[cfg(target_os = "linux")]
#[no_mangle]
extern "system" fn ModuleExit() -> bool {
    true
}

#[no_mangle]
extern "system" fn GetPluginFactory() -> *mut IPluginFactory {
    ComWrapper::new(Factory {})
        .to_com_ptr::<IPluginFactory>()
        .unwrap()
        .into_raw()
}
