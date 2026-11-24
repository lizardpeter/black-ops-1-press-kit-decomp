#include "presskit/navigation.hpp"
#include <array>
#include <stdexcept>
namespace presskit {
// Frame 75: five distinct onRelease handlers at 2501236, 2501276,
// 2501313, 2501351 and 2501407.
NavigationRequest welcome_release(WelcomeControl control) {
    switch(control) {
    case WelcomeControl::wallpaper: return {NavigationKind::play_from_label,"Wallpaper",""};
    case WelcomeControl::videos: return {NavigationKind::play_from_label,"Videos",""};
    case WelcomeControl::screens: return {NavigationKind::play_from_label,"Screens",""};
    case WelcomeControl::website: return {NavigationKind::open_url,"http://www.callofduty.com","_blank"};
    case WelcomeControl::close: return {NavigationKind::quit,"",""};
    }
    throw std::invalid_argument("unknown host control");
}
// Frame 81: wp1 through wp8, original button order, not sorted resolution order.
std::optional<NavigationRequest> wallpaper_release(int button) {
    static constexpr std::array<std::string_view,8> paths{
        "payload/wallpaper/1024x768/COD7-Wallpaper-1024x768.jpg",
        "payload/wallpaper/1280x800/COD7-Wallpaper-1280x800.jpg",
        "payload/wallpaper/1280x1024/COD7-Wallpaper-1280x1024.jpg",
        "payload/wallpaper/1440x900/COD7-Wallpaper-1440x900.jpg",
        "payload/wallpaper/1680x1050/COD7-Wallpaper-1680x1050.jpg",
        "payload/wallpaper/1600x1200/COD7-Wallpaper-1600x1200.jpg",
        "payload/wallpaper/1920x1200/COD7-Wallpaper-1920x1200.jpg",
        "payload/wallpaper/2560x1440/COD7-Wallpaper-2560x1440.jpg"
    };
    if(button<1 || button>8) return std::nullopt;
    return NavigationRequest{NavigationKind::open_url,paths[button-1],""};
}
// Frame 91: trl_1 through trl_6; timeline play, assign vid, stop music.
bool VideoMenu::release(int button) {
    if(button<1 || button>6) return false;
    timeline_playing=true;
    vid=button;
    music_stopped=true;
    return true;
}
// Frame 95 at 2726848..2727132: independent equality branches.
// Original has no assignment/play for vid 3. Do not invent a missing asset.
std::optional<std::string_view> video_path(int vid) {
    switch(vid) {
    case 1:return "payload/movies/1.BlackOpsMPTeaser.mp4";
    case 2:return "payload/movies/2.PrestigeEditionVideo.mp4";
    case 4:return "payload/movies/4.UncutRevealTrailer.mp4";
    case 5:return "payload/movies/5.DebutTeaseTrailer.mp4";
    case 6:return "payload/movies/6.Wagermatch.mp4";
    default:return std::nullopt;
    }
}
// Frame 95: resetCounter 2726255 and checkCounter 2726133.
// Exact equality at 50, not >=; interval cancellation modeled as a host effect.
void CursorCounter::reset() { counter=0; }
void CursorCounter::check() {
    ++counter;
    if(counter==50) {
        reset();
        interval_active=false;
        mouse_visible=false;
        vmouse=0;
    }
}
// Frame 95 2726667..2726727 and 2727133..2727355.
VideoLayout video_layout(double root_w,double root_h,double stage_width) {
    const double video_w=stage_width;
    const double video_h=stage_width*0.5625;
    return {0-(root_w-1280)/2+(root_w-video_w)/2,
            0-(root_h-720)/2+(root_h-video_h)/2,
            video_w, video_h,
            root_h-(root_h-720)/2-100};
}
}
