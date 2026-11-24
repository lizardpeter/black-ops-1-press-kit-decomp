#include "presskit/navigation.hpp"
#include <cstdlib>
using namespace presskit;
static void check(bool b){if(!b)std::abort();}
int main() {
    auto n=welcome_release(WelcomeControl::wallpaper);
    check(n.kind==NavigationKind::play_from_label && n.value=="Wallpaper");
    check(welcome_release(WelcomeControl::videos).value=="Videos");
    check(welcome_release(WelcomeControl::screens).value=="Screens");
    n=welcome_release(WelcomeControl::website);
    check(n.value=="http://www.callofduty.com" && n.target=="_blank");
    check(welcome_release(WelcomeControl::close).kind==NavigationKind::quit);
    check(!wallpaper_release(0) && !wallpaper_release(9));
    for(int i=1;i<=8;++i)check(wallpaper_release(i)->kind==NavigationKind::open_url);
    check(wallpaper_release(2)->value=="payload/wallpaper/1280x800/COD7-Wallpaper-1280x800.jpg");
    check(wallpaper_release(5)->value=="payload/wallpaper/1680x1050/COD7-Wallpaper-1680x1050.jpg");
    check(wallpaper_release(6)->value=="payload/wallpaper/1600x1200/COD7-Wallpaper-1600x1200.jpg");
    VideoMenu v; check(!v.release(0));check(!v.release(7));
    for(int i=1;i<=6;++i){check(v.release(i) && v.vid==i && v.timeline_playing && v.music_stopped);}
    for(int i:{1,2,4,5,6}) check(video_path(i).has_value());
    check(!video_path(3));check(!video_path(0));check(!video_path(7));
    CursorCounter c; for(int i=0;i<49;++i)c.check();
    check(c.counter==49 && c.mouse_visible && c.interval_active);
    c.check();check(c.counter==0 && !c.interval_active && !c.mouse_visible && c.vmouse==0);
    c.counter=50;c.check();check(c.counter==51);
    auto l=video_layout(1920,1080,1920);
    check(l.width==1920 && l.height==1080 && l.x==-320 && l.y==-180 && l.control_y==800);
    l=video_layout(1280,720,1280);
    check(l.width==1280 && l.height==720 && l.x==0 && l.y==0 && l.control_y==620);
}
