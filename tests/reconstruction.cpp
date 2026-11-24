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
}
