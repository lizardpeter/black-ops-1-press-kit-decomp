#pragma once
#include "presskit/gallery.hpp"
#include "presskit/media.hpp"
#include "presskit/launcher.hpp"
#include <vector>
namespace presskit {
// Runtime orchestration of decoded root timeline, not a general AVM1 VM.
enum class EffectKind { play_intro, play_video, stop_video, load_screens_xml, start_music, stop_music, open_url, quit };
struct Effect { EffectKind kind; std::string_view value{}; std::string_view target{}; };
enum class Scene { intro, transition, welcome, wallpaper, videos, video_player, screens, gallery };
struct Timeline {
    static constexpr double fps=24;
    int frame=1;
    bool playing=false, rewinding=false, nav_visible=false, close_visible=false;
    bool mouse_visible=false;
    Gallery gallery;
    VideoMenu video;
    MediaControls controls;
    std::vector<Effect> effects;
    Timeline();
    Scene scene() const;
    void tick();
    bool navigate(WelcomeControl control);
    bool select_video(int button);
    bool select_wallpaper(int button);
    bool select_screenshot(int button);
    bool intro_complete();
    bool close_media();
    bool video_complete();
    std::vector<Effect> take_effects();
private:
    void enter(int f);
    void start_from(int label);
};
struct ControlOrigin { int id; double x,y; };
std::optional<ControlOrigin> trailer_origin(int id);
std::optional<ControlOrigin> wallpaper_origin(int id);
std::optional<ControlOrigin> screenshot_origin(int id);
}
