#pragma once
#include <optional>
#include <string_view>
namespace presskit {
enum class WelcomeControl { wallpaper, videos, screens, website, close };
enum class NavigationKind { play_from_label, open_url, quit };
struct NavigationRequest {
    NavigationKind kind;
    std::string_view value;
    std::string_view target;
};
NavigationRequest welcome_release(WelcomeControl control);
std::optional<NavigationRequest> wallpaper_release(int button);
struct VideoMenu {
    int vid=0;
    bool timeline_playing=false, music_stopped=false;
    bool release(int button);
};
std::optional<std::string_view> video_path(int vid);
struct CursorCounter {
    double counter=0;
    bool interval_active=true, mouse_visible=true;
    int vmouse=1;
    void reset();
    void check();
};
struct VideoLayout { double x,y,width,height,control_y; };
VideoLayout video_layout(double root_w,double root_h,double stage_width);
}
