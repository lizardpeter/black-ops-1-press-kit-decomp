#include "presskit/launcher.hpp"
#include "presskit/gallery.hpp"
#include <cstdlib>
using namespace presskit;
void check(bool pass) { if (!pass) std::abort(); }
int main() {
    auto p=shell_position(1920,1080); check(p.x==320 && p.y==180);
    p=shell_position(1280,720); check(p.x==0 && p.y==0);
    p=shell_position(1279,719); check(p.x==-0.5 && p.y==-0.5);
    Launcher l; check(l.no_visible && l.yes_visible && l.text_visible && l.launch_visible);
    l.accept(); check(l.loaded_movie=="COD7-DVD.swf" && l.fullscreen_requested);
    check(!l.no_visible && !l.yes_visible && !l.text_visible && !l.launch_visible && !l.quit_requested);
    Launcher n; n.decline(); check(n.quit_requested && n.loaded_movie.empty() && !n.fullscreen_requested);
    check(picture_num(0,20)=="1 / 20"); check(picture_num(19,20)=="20 / 20");
    Gallery g; g.total=20; check(!g.previous());
    auto req=g.first(); check(req && req->index==0 && req->alpha==1 && req->load_argument==1);
    g.filesize=100; g.loaded=20;
    check(!g.next() && g.index==1); check(!g.first());
    req=g.previous(); check(req && req->index==0 && req->counter=="1 / 20");
    g.loaded=100; req=g.next(); check(req && req->index==1 && req->counter=="2 / 20");
    g.index=19; check(!g.next() && g.index==19);
    req=g.first(); check(req && req->index==19);
}
