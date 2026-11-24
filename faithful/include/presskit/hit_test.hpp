#pragma once
#include "presskit/timeline.hpp"
namespace presskit {
enum class HitKind { none, accept, decline, wallpaper_menu, videos_menu, screens_menu, website, quit, wallpaper, trailer, thumbnail, close_media, previous, next, play_pause, seek };
struct Hit { HitKind kind=HitKind::none; int id=0; };
Hit launcher_hit(double x,double y);
Hit menu_hit(Scene scene,double x,double y);
Hit media_hit(Scene scene,double x,double y,double controls_y);
}
