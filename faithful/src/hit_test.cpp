#include "presskit/hit_test.hpp"
namespace presskit {
static bool inside(double x,double y,double left,double top,double w,double h){
    return x>=left&&x<left+w&&y>=top&&y<top+h;
}
// Launcher button 8: hit shape 7, [0,188]×[0,54], placements 48182/48202.
Hit launcher_hit(double x,double y){
    if(inside(x,y,49,199,188,54))return {HitKind::accept};
    if(inside(x,y,259,199,188,54))return {HitKind::decline};
    return {};
}
// Nominal stage pixels. AABB of original hit-state shapes; exact vector
// contours not yet evaluated. Nav at (223,23) on frame 75.
Hit menu_hit(Scene scene,double x,double y){
    if(scene!=Scene::welcome&&scene!=Scene::wallpaper&&scene!=Scene::videos&&scene!=Scene::screens)return {};
    if(inside(x,y,1091.5,26.5,16.05,16.05))return {HitKind::quit};
    // Button 36/39/42 hit shape 25, transformed by button matrix.
    constexpr double sx=97784.0/65536, sy=228319.0/65536;
    const double nx=233+628*sx-937, w=126*sx, h=31*sy;
    if(inside(x,y,nx,23+7+108,w,h))return {HitKind::wallpaper_menu};
    if(inside(x,y,nx,23+129+108,w,h))return {HitKind::videos_menu};
    if(inside(x,y,nx,23+251+108,w,h))return {HitKind::screens_menu};
    if(inside(x,y,233,542,188,54))return {HitKind::website};
    if(scene==Scene::wallpaper)for(int i=1;i<=8;++i){
        auto o=wallpaper_origin(i);if(inside(x,y,o->x,o->y,87,29))return {HitKind::wallpaper,i};
    }
    if(scene==Scene::videos)for(int i=1;i<=6;++i){
        auto o=trailer_origin(i);if(o&&inside(x,y,o->x,o->y,172,96))return {HitKind::trailer,i};
    }
    if(scene==Scene::screens)for(int i=1;i<=20;++i){
        auto o=screenshot_origin(i);if(inside(x,y,o->x,o->y,102,90))return {HitKind::thumbnail,i};
    }
    return {};
}
Hit media_hit(Scene scene,double x,double y,double cy){
    if(scene!=Scene::video_player&&scene!=Scene::gallery)return {};
    if(inside(x,y,961,cy,102.95,31))return {HitKind::close_media};
    if(scene==Scene::gallery){
        if(inside(x,y,215,cy,112,31))return {HitKind::previous};
        if(inside(x,y,365,cy,114.6,31))return {HitKind::next};
    }else{
        // Adobe component buttons lack authored DefineButton2 hit bounds.
        // Provisional native control area, not asserted as original.
        if(inside(x,y,215,cy,112,31))return {HitKind::play_pause};
        if(inside(x,y,241,cy+48,785,24))return {HitKind::seek};
    }
    return {};
}
}
