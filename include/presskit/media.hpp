#pragma once
#include "presskit/navigation.hpp"
#include <string>
namespace presskit {
struct Rect { double x,y,width,height; };
struct IntroLayout { Rect movie, overlay; };
IntroLayout intro_layout(double root_w,double root_h,double stage_width);
Rect background_layout(double root_w,double root_h,double stage_width);
inline constexpr std::string_view intro_path="payload/intro/IntroSeq.mp4";
struct MediaControls {
    CursorCounter cursor;
    double target_y=0;
    bool rewind_handler=false;
    void mouse_activity();
    void enter_frame(double root_h);
    void close_or_complete();
};
double slide_step(double current,double target);
std::string flash_time(double seconds);
struct VideoControlFrame { double y; std::string display; };
VideoControlFrame video_control_frame(double current_y,double target_y,double playhead,double total);
struct GalleryControlFrame { double y, mouse_y; };
GalleryControlFrame gallery_control_frame(double current_y,double target_y,double stage_mouse_y,double root_h);
struct ImageLoader {
    double loaded=0, filesize=0, alpha=1, scale=0;
    bool visible=false;
    void enter_frame(double bytes_loaded,double bytes_total);
};
}
