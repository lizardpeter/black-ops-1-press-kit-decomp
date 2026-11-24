#include "presskit/media.hpp"
#include "presskit/gallery.hpp"
#include <cstdlib>
#include <cmath>
#include <limits>
using namespace presskit;
static void check(bool b){if(!b)std::abort();}
int main(){
    auto i=intro_layout(1920,1080,1280);
    check(i.movie.x==0 && i.movie.y==0 && i.movie.height==720);
    check(i.overlay.x==-320 && i.overlay.y==0);
    auto b=background_layout(1920,1080,1920);
    check(b.x==-320 && b.y==-180 && b.width==1920 && b.height==1080);
    MediaControls c;c.cursor.vmouse=0;c.cursor.mouse_visible=false;
    c.enter_frame(720);check(c.target_y==770);
    c.mouse_activity();check(c.cursor.mouse_visible && c.cursor.vmouse==1 && c.cursor.counter==0);
    for(int n=0;n<49;++n)c.enter_frame(720);
    check(c.cursor.counter==49 && c.target_y==620);
    c.enter_frame(720);check(c.cursor.vmouse==0 && c.target_y==620);
    c.enter_frame(720);check(c.target_y==770);
    c.close_or_complete();check(c.rewind_handler);
    check(slide_step(770,620)==740);
    check(slide_step(620,770)==650);
    check(slide_step(619,620)==619);
    check(flash_time(59.49)=="00:59" && flash_time(59.5)=="01:00");
    check(flash_time(600)=="10:00");
    check(flash_time(std::numeric_limits<double>::quiet_NaN())=="NaN:NaN");
    auto v=video_control_frame(770,620,62.1,125.4);
    check(v.y==740 && v.display=="01:02/02:05");
    auto g=gallery_control_frame(770,620,55,1080);
    check(g.y==740 && g.mouse_y==235);
    ImageLoader l;l.enter_frame(20,100);check(l.visible && l.scale==20 && l.alpha==1);
    l.enter_frame(100,100);check(!l.visible && l.alpha==6 && l.scale==20);
    l.alpha=499;l.enter_frame(100,100);check(l.alpha==504);
    l.enter_frame(100,100);check(l.alpha==504);
    l.enter_frame(0,0);check(!l.visible);
    Gallery gallery;gallery.total=3;
    check(!gallery.key_down(37));check(gallery.key_down(39)->index==1);
    check(!gallery.key_down(40));check(gallery.previous_release()->index==0);
    check(gallery.next_release()->index==1);
}
