#include "presskit/timeline.hpp"
#include <utility>
namespace presskit {
Timeline::Timeline(){enter(1);}
Scene Timeline::scene() const {
    switch(frame){
    case 1:return Scene::intro;
    case 75:return Scene::welcome;
    case 81:case 85:return Scene::wallpaper;
    case 91:return Scene::videos;
    case 95:return Scene::video_player;
    case 101:return Scene::screens;
    case 105:return Scene::gallery;
    default:return Scene::transition;
    }
}
void Timeline::enter(int f) {
    frame=f;
    switch(frame){
    case 1:
        playing=false;mouse_visible=false;
        effects.push_back({EffectKind::play_intro,intro_path});break;
    case 75:
        playing=false;mouse_visible=true;nav_visible=close_visible=true;break;
    case 81:case 85:playing=false;break;
    case 91:
        playing=false;rewinding=false;controls.rewind_handler=false;
        effects.push_back({EffectKind::start_music});break;
    case 94:case 104:nav_visible=close_visible=true;break;
    case 95:
        playing=false;nav_visible=close_visible=mouse_visible=false;
        controls.cursor.vmouse=0;controls.cursor.mouse_visible=false;
        if(auto p=video_path(video.vid))effects.push_back({EffectKind::play_video,*p});
        break;
    case 101:playing=false;rewinding=false;controls.rewind_handler=false;break;
    case 105:
        playing=false;nav_visible=close_visible=mouse_visible=false;
        controls.cursor.vmouse=0;controls.cursor.mouse_visible=false;
        effects.push_back({EffectKind::load_screens_xml,"payload/screens/screens.xml"});break;
    }
}
void Timeline::tick(){
    if(rewinding){
        // stopRoll2 (frame 91 or 101) calls prevFrame; the control instance
        // persists until the menu frame, where it is removed from the timeline.
        playing=false;enter(frame-1);
    }else if(playing && frame<105)enter(frame+1);
}
void Timeline::start_from(int label){playing=true;rewinding=false;enter(label);}
bool Timeline::navigate(WelcomeControl control){
    if(!nav_visible||scene()==Scene::transition)return false;
    switch(control){
    case WelcomeControl::wallpaper:start_from(76);break;
    case WelcomeControl::videos:start_from(86);break;
    case WelcomeControl::screens:start_from(96);break;
    case WelcomeControl::website:{auto r=welcome_release(control);effects.push_back({EffectKind::open_url,r.value,r.target});break;}
    case WelcomeControl::close:effects.push_back({EffectKind::quit});break;
    }
    return true;
}
bool Timeline::select_video(int button){
    if(scene()!=Scene::videos || !trailer_origin(button) || !video.release(button))return false;
    playing=true;effects.push_back({EffectKind::stop_music});return true;
}
bool Timeline::select_wallpaper(int button){
    if(scene()!=Scene::wallpaper)return false;
    auto r=wallpaper_release(button);
    if(!r)return false;
    effects.push_back({EffectKind::open_url,r->value,r->target});return true;
}
bool Timeline::select_screenshot(int button){
    if(scene()!=Scene::screens || !gallery.select_thumbnail(button))return false;
    playing=true;return true;
}
bool Timeline::intro_complete(){
    if(frame!=1)return false;
    playing=true;return true;
}
bool Timeline::close_media(){
    if(scene()!=Scene::video_player && scene()!=Scene::gallery)return false;
    if(rewinding)return false;
    controls.close_or_complete();rewinding=true;
    // Host releases video when the placed video instance is removed.
    if(scene()==Scene::video_player)effects.push_back({EffectKind::stop_video});
    return true;
}
bool Timeline::video_complete(){return scene()==Scene::video_player && close_media();}
std::vector<Effect> Timeline::take_effects(){return std::exchange(effects,{});}
// Native positions are placement origins, not clickable bounds.
// Sprite 157 at 2724519..2724619; trl_3 is not placed.
std::optional<ControlOrigin> trailer_origin(int id){
    switch(id){
    case 1:return ControlOrigin{id,533,208};
    case 2:return ControlOrigin{id,712,208};
    case 4:return ControlOrigin{id,891,208};
    case 5:return ControlOrigin{id,533,381};
    case 6:return ControlOrigin{id,712,381};
    default:return std::nullopt;
    }
}
// Sprite 136, root placement (526.85,43), 2589453..2589593.
std::optional<ControlOrigin> wallpaper_origin(int id){
    if(id<1||id>8)return std::nullopt;
    constexpr double x[]{6,154,301,449};
    return ControlOrigin{id,526.85+x[(id-1)/2],43.0+(id%2?444:480)};
}
// Sprite 220, root placement (526.85,43), 3140695..3141142.
std::optional<ControlOrigin> screenshot_origin(int id){
    if(id<1||id>20)return std::nullopt;
    return ControlOrigin{id,533.0+107*((id-1)%5),158.0+95*((id-1)/5)};
}
}
