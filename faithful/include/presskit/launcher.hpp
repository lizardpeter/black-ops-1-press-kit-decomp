#pragma once
#include <string_view>
namespace presskit {
struct Position { double x, y; };
Position shell_position(double width, double height);
struct Launcher {
    bool no_visible=true, yes_visible=true, text_visible=true, launch_visible=true;
    bool fullscreen_requested=false, quit_requested=false;
    std::string_view loaded_movie{};
    void accept();
    void decline();
};
}
