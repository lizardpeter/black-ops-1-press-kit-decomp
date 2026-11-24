#include "presskit/timeline.hpp"
#include <cstdlib>
#include <cmath>
using namespace presskit;
static void check(bool b){if(!b)std::abort();}
static void ticks(Timeline& t,int n){while(n--)t.tick();}
int main(){
    Timeline t;
    auto e=t.take_effects();check(e.size()==1&&e[0].kind==EffectKind::play_intro);
    ticks(t,50);check(t.frame==1&&!t.navigate(WelcomeControl::videos));
    check(t.intro_complete());ticks(t,74);
    check(t.frame==75&&t.scene()==Scene::welcome&&!t.playing&&t.nav_visible);
    check(t.navigate(WelcomeControl::wallpaper));check(t.frame==76);ticks(t,5);
    check(t.frame==81&&t.scene()==Scene::wallpaper);
    check(t.select_wallpaper(5));e=t.take_effects();check(e.size()==1&&e[0].kind==EffectKind::open_url);
    check(t.navigate(WelcomeControl::videos));ticks(t,5);
    check(t.frame==91&&t.scene()==Scene::videos);
    e=t.take_effects();check(e.size()==1&&e[0].kind==EffectKind::start_music);
    check(!t.select_video(3));check(t.select_video(4));
    ticks(t,4);check(t.frame==95&&!t.nav_visible&&!t.playing);
    e=t.take_effects();check(e.size()==2&&e[0].kind==EffectKind::stop_music&&e[1].value=="payload/movies/4.UncutRevealTrailer.mp4");
    check(t.video_complete());t.tick();check(t.frame==94&&t.nav_visible);
    ticks(t,3);check(t.frame==91&&!t.rewinding&&!t.playing);
    e=t.take_effects();check(e.size()==2&&e[0].kind==EffectKind::stop_video&&e[1].kind==EffectKind::start_music);
    check(t.navigate(WelcomeControl::screens));ticks(t,5);
    check(t.frame==101&&t.select_screenshot(20));ticks(t,4);
    check(t.frame==105&&t.gallery.index==19);
    e=t.take_effects();check(e.size()==1&&e[0].kind==EffectKind::load_screens_xml);
    check(t.close_media());ticks(t,4);check(t.frame==101&&!t.rewinding&&t.nav_visible);
    check(trailer_origin(6)->x==712&&!trailer_origin(3));
    check(screenshot_origin(20)->x==961&&screenshot_origin(20)->y==443);
    check(std::abs(wallpaper_origin(8)->x-975.85)<0.00001);
}
