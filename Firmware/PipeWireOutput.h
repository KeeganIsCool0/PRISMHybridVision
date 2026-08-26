#pragma once
#include "AudioEngine.h"
#include <memory>
namespace spatial { class PipeWireOutput { public: struct Impl; explicit PipeWireOutput(AudioEngine&); ~PipeWireOutput(); PipeWireOutput(const PipeWireOutput&)=delete; bool start(); void stop(); private: std::unique_ptr<Impl> impl_; }; }
