#include "PipeWireOutput.h"
#include <pipewire/pipewire.h>
#include <spa/param/audio/format-utils.h>
#include <spa/pod/builder.h>
#include <cmath>

namespace spatial {
struct PipeWireOutput::Impl {
    AudioEngine& engine;
    pw_thread_loop* loop=nullptr;
    pw_stream* stream=nullptr;
    pw_stream_events events{}; // Must outlive pw_stream: PipeWire retains this table.
    float phase=0;
    explicit Impl(AudioEngine& e):engine(e){}
};
static void process(void* data) {
    auto& i=*static_cast<PipeWireOutput::Impl*>(data); auto* b=pw_stream_dequeue_buffer(i.stream); if(!b||!b->buffer->datas[0].data) return;
    auto& d=b->buffer->datas[0]; const auto frames=d.chunk->size/(sizeof(float)*kOutputChannels); auto* packed=static_cast<float*>(d.data)+d.chunk->offset/sizeof(float); std::vector<float> mono(frames); std::array<std::vector<float>,kOutputChannels> channels; std::array<float*,kOutputChannels> ptrs;
    for(size_t n=0;n<frames;++n) { mono[n]=0.10F*std::sin(i.phase); i.phase+=2*3.14159265F*440/48000; if(i.phase>6.2831853F)i.phase-=6.2831853F; }
    for(size_t c=0;c<kOutputChannels;++c){channels[c].resize(frames);ptrs[c]=channels[c].data();} i.engine.renderMono(mono.data(),frames,ptrs.data());
    for(size_t n=0;n<frames;++n)
        for(size_t c=0;c<kOutputChannels;++c)
            packed[n*kOutputChannels+c]=channels[c][n];
    d.chunk->offset=0; d.chunk->stride=sizeof(float)*kOutputChannels;
    d.chunk->size=frames*d.chunk->stride;
    pw_stream_queue_buffer(i.stream,b);
}
PipeWireOutput::PipeWireOutput(AudioEngine& e):impl_(std::make_unique<Impl>(e)){}
PipeWireOutput::~PipeWireOutput(){stop();}
bool PipeWireOutput::start() {
    pw_init(nullptr,nullptr);
    impl_->loop=pw_thread_loop_new("spatial-panner",nullptr);
    if(!impl_->loop) return false;
    auto* l=pw_thread_loop_get_loop(impl_->loop);
    impl_->events.version=PW_VERSION_STREAM_EVENTS;
    impl_->events.process=process;
    impl_->stream=pw_stream_new_simple(l,"9.2.6 Spatial Panner",pw_properties_new(
        PW_KEY_MEDIA_TYPE,"Audio",PW_KEY_MEDIA_CATEGORY,"Playback",PW_KEY_MEDIA_ROLE,"Production",
        PW_KEY_NODE_DESCRIPTION,"17-channel 9.2.6 spatial bed",nullptr),&impl_->events,impl_.get());
    spa_audio_info_raw info{}; info.format=SPA_AUDIO_FORMAT_F32; info.rate=48000; info.channels=kOutputChannels;
    uint8_t buffer[1024]; spa_pod_builder builder=SPA_POD_BUILDER_INIT(buffer,sizeof(buffer));
    const spa_pod* params[]={spa_format_audio_raw_build(&builder,SPA_PARAM_EnumFormat,&info)};
    if(pw_stream_connect(impl_->stream,PW_DIRECTION_OUTPUT,PW_ID_ANY,
        static_cast<pw_stream_flags>(PW_STREAM_FLAG_AUTOCONNECT|PW_STREAM_FLAG_MAP_BUFFERS|PW_STREAM_FLAG_RT_PROCESS),params,1)<0) return false;
    return pw_thread_loop_start(impl_->loop)==0;
}
void PipeWireOutput::stop(){
    // A stream must be disconnected while its owning loop is locked.  Destroying
    // a still-connected proxy after stopping the loop can race its server reply.
    if(impl_->loop) pw_thread_loop_lock(impl_->loop);
    if(impl_->stream) pw_stream_disconnect(impl_->stream);
    if(impl_->loop) pw_thread_loop_unlock(impl_->loop);
    if(impl_->loop) pw_thread_loop_stop(impl_->loop);
    if(impl_->loop) pw_thread_loop_lock(impl_->loop);
    if(impl_->stream){ pw_stream_destroy(impl_->stream); impl_->stream=nullptr; }
    if(impl_->loop) pw_thread_loop_unlock(impl_->loop);
    if(impl_->loop){pw_thread_loop_destroy(impl_->loop);impl_->loop=nullptr;}
    pw_deinit();
}
} // namespace spatial
