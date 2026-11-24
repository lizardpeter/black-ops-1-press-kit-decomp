#include "presskit/launcher.hpp"
namespace presskit {
// launcher 1529, 104-byte AVM1 body: division by 2 then subtraction.
Position shell_position(double width, double height) {
    return {width / 2.0 - 640.0, height / 2.0 - 360.0};
}
// launcher 1656: observable effects; actual Flash host calls belong to integration.
void Launcher::accept() {
    loaded_movie = "COD7-DVD.swf";
    fullscreen_requested = true;
    no_visible = yes_visible = text_visible = launch_visible = false;
}
// launcher 1781: FSCommand quit.
void Launcher::decline() { quit_requested = true; }
}
