#include "presskit/media.hpp"
#include <cmath>
#include <sstream>
#include <iomanip>
namespace presskit {
// Frame 1 486012..486422: intro and video11 overlay differ in x positioning.
IntroLayout intro_layout(double root_w,double root_h,double stage_width) {
    auto v=video_layout(root_w,root_h,stage_width);
    return {{v.x,v.y,v.width,v.height},{0-(root_w-1280)/2,v.y,v.width,v.height}};
}
// Frame 56 at 882042: BG is sized and centered like intro.
Rect background_layout(double root_w,double root_h,double stage_width) {
    auto v=video_layout(root_w,root_h,stage_width);
    return {v.x,v.y,v.width,v.height};
}
// 2726385/2726447 and 3143953/3144015.
void MediaControls::mouse_activity() {
    cursor.mouse_visible=true;cursor.vmouse=1;cursor.counter=0;
}
// 2726515 and 3144083: test Vmouse before calling checkCounter.
// When checkCounter hides the mouse, the visible target remains for that tick.
void MediaControls::enter_frame(double root_h) {
    const double edge=root_h-(root_h-720)/2;
    if(cursor.vmouse==1) {
        cursor.check();
        target_y=edge-100;
    } else target_y=edge+50;
}
// 2726745, 2726798, 3144313: install stopRoll2, rather than rewind immediately.
void MediaControls::close_or_complete() { rewind_handler=true; }
// 2727367 and 3144409, including asymmetrical ceil easing.
double slide_step(double current,double target) {
    return current-std::ceil((current-target)/5);
}
// Frame 95: Flash Math.round, floor, remainder, pad each component <10.
// Media duration domain is nonnegative finite seconds; unknown metadata is NaN.
std::string flash_time(double seconds) {
    auto numeric=[](double x) {
        if(std::isnan(x)) return std::string("NaN");
        if(std::isinf(x)) return std::string(x<0?"-Infinity":"Infinity");
        std::ostringstream s;s<<std::fixed<<std::setprecision(0)<<(x==0?0:x);return s.str();
    };
    const double rounded=std::floor(seconds+0.5);
    const double minutes=std::floor(rounded/60);
    const double secs=std::floor(std::fmod(rounded,60));
    auto pad=[&](double x){return (x<10?"0":"")+numeric(x);};
    return pad(minutes)+":"+pad(secs);
}
VideoControlFrame video_control_frame(double current_y,double target_y,double playhead,double total) {
    return {slide_step(current_y,target_y),flash_time(playhead)+"/"+flash_time(total)};
}
GalleryControlFrame gallery_control_frame(double current_y,double target_y,double stage_mouse_y,double root_h) {
    return {slide_step(current_y,target_y),stage_mouse_y+(root_h-720)/2};
}
// 3145040: original compares alpha to 500, not 100; native Flash host clamping
// is not imposed on this raw requested alpha model.
void ImageLoader::enter_frame(double bytes_loaded,double bytes_total) {
    filesize=bytes_total;loaded=bytes_loaded;visible=true;
    if(loaded!=filesize) scale=100*loaded/filesize;
    else {visible=false;if(alpha<500)alpha+=5;}
}
}
