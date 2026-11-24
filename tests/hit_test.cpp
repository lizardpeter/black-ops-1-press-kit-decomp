#include "presskit/hit_test.hpp"
#include <cstdlib>
using namespace presskit;
static void check(bool b){if(!b)std::abort();}
int main(){
 check(launcher_hit(50,200).kind==HitKind::accept);
 check(launcher_hit(260,200).kind==HitKind::decline);
 check(launcher_hit(249,200).kind==HitKind::none);
 check(menu_hit(Scene::welcome,250,150).kind==HitKind::wallpaper_menu);
 check(menu_hit(Scene::welcome,250,270).kind==HitKind::videos_menu);
 check(menu_hit(Scene::welcome,250,390).kind==HitKind::screens_menu);
 check(menu_hit(Scene::videos,900,220).id==4);
 check(menu_hit(Scene::videos,900,390).kind==HitKind::none);
 check(menu_hit(Scene::screens,962,444).id==20);
 check(menu_hit(Scene::wallpaper,977,524).id==8);
 check(media_hit(Scene::gallery,216,621,620).kind==HitKind::previous);
 check(media_hit(Scene::gallery,366,621,620).kind==HitKind::next);
 check(media_hit(Scene::gallery,962,621,620).kind==HitKind::close_media);
}
