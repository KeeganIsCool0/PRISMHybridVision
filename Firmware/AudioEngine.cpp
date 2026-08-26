#include "AudioEngine.h"

#include <algorithm>
#include <cmath>
#include <numbers>

namespace spatial {
namespace {
constexpr float kPi = std::numbers::pi_v<float>;
constexpr float kSampleRate = 48000.0F;
constexpr std::array<Position, kOutputChannels> kSpeakerPositions{{
    {-30,0,1}, {0,0,1}, {30,0,1}, {-60,0,1}, {60,0,1}, {-100,0,1}, {100,0,1}, {-145,0,1}, {145,0,1},
    {-30,0,1}, {30,0,1}, {-30,45,1}, {30,45,1}, {-90,55,1}, {90,55,1}, {-150,45,1}, {150,45,1}
}};
float radians(float d) { return d * kPi / 180.0F; }
std::array<float,3> vector(Position p) {
    const auto e=radians(p.elevationDegrees), a=radians(p.azimuthDegrees);
    return {std::cos(e)*std::sin(a), std::cos(e)*std::cos(a), std::sin(e)};
}
} // namespace

float AudioEngine::Biquad::process(float x) { const float y=b0*x+z1; z1=b1*x-a1*y+z2; z2=b2*x-a2*y; return y; }
AudioEngine::Biquad AudioEngine::shelf(float db, float frequency, bool high) {
    const float A=std::pow(10.0F,db/40.0F), w=2*kPi*frequency/kSampleRate;
    const float c=std::cos(w), s=std::sin(w), alpha=s/2.0F*std::sqrt(2.0F), beta=2*std::sqrt(A)*alpha;
    Biquad q; float a0;
    if (!high) { q.b0=A*((A+1)-(A-1)*c+beta); q.b1=2*A*((A-1)-(A+1)*c); q.b2=A*((A+1)-(A-1)*c-beta); a0=(A+1)+(A-1)*c+beta; q.a1=-2*((A-1)+(A+1)*c); q.a2=(A+1)+(A-1)*c-beta; }
    else { q.b0=A*((A+1)+(A-1)*c+beta); q.b1=-2*A*((A-1)+(A+1)*c); q.b2=A*((A+1)+(A-1)*c-beta); a0=(A+1)-(A-1)*c+beta; q.a1=2*((A-1)-(A+1)*c); q.a2=(A+1)-(A-1)*c-beta; }
    q.b0/=a0; q.b1/=a0; q.b2/=a0; q.a1/=a0; q.a2/=a0; return q;
}
AudioEngine::AudioEngine() { std::scoped_lock lock(mutex_); refreshGainsLocked(); refreshFiltersLocked(); }
void AudioEngine::setPosition(Position p) { std::scoped_lock lock(mutex_); position_.azimuthDegrees=std::clamp(p.azimuthDegrees,-180.0F,180.0F); position_.elevationDegrees=std::clamp(p.elevationDegrees,-30.0F,90.0F); position_.distance=std::clamp(p.distance,0.25F,20.0F); refreshGainsLocked(); }
Position AudioEngine::position() const { std::scoped_lock lock(mutex_); return position_; }
void AudioEngine::setSpread(float d) { std::scoped_lock lock(mutex_); spreadDegrees_=std::clamp(d,5.0F,180.0F); refreshGainsLocked(); }
void AudioEngine::setRoomCorrection(Channel c, RoomCorrection r) { std::scoped_lock lock(mutex_); filters_[static_cast<size_t>(c)].settings=r; refreshFiltersLocked(); }
std::array<float,kOutputChannels> AudioEngine::gains() const { std::scoped_lock lock(mutex_); return gains_; }
void AudioEngine::refreshGainsLocked() {
    gains_.fill(0); const auto source=vector(position_); float sum=0;
    for (size_t i=0;i<kOutputChannels;++i) { if (i==9||i==10) continue; const auto s=vector(kSpeakerPositions[i]); const float dot=source[0]*s[0]+source[1]*s[1]+source[2]*s[2]; const float angle=std::acos(std::clamp(dot,-1.0F,1.0F))*180.0F/kPi; gains_[i]=std::exp(-0.5F*(angle/spreadDegrees_)*(angle/spreadDegrees_)); sum+=gains_[i]*gains_[i]; }
    const float normalization=sum>0 ? 1/std::sqrt(sum) : 1; for(size_t i=0;i<kOutputChannels;++i) gains_[i]*=normalization/position_.distance;
    // Deliberately low, fixed LFE send: LFE is non-directional.
    gains_[9]=gains_[10]=0.158F/position_.distance;
}
void AudioEngine::refreshFiltersLocked() { for(auto& f:filters_) { f.low=shelf(f.settings.lowShelfDb,120,false); f.high=shelf(f.settings.highShelfDb,6000,true); } }
void AudioEngine::renderMono(const float* in,size_t frames,float* const* out) { std::scoped_lock lock(mutex_); for(size_t c=0;c<kOutputChannels;++c) for(size_t n=0;n<frames;++n) { float x=in[n]*gains_[c]; auto& f=filters_[c]; out[c][n]=f.settings.bypass?x:f.low.process(f.high.process(x))*std::pow(10.0F,f.settings.gainDb/20.0F); } }
std::string_view AudioEngine::channelName(Channel c) { static constexpr std::array<std::string_view,kOutputChannels> n{"FL","FC","FR","WL","WR","SL","SR","BL","BR","LFE1","LFE2","TFL","TFR","TML","TMR","TBL","TBR"}; return n[static_cast<size_t>(c)]; }
} // namespace spatial
