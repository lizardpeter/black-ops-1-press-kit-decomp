#include "presskit/background.hpp"
#include <cstdlib>
using namespace presskit;
static void check(bool b){if(!b)std::abort();}
int main(){
    Background b;b.advance(42);check(b.frame==2&&!b.playing&&b.interval_active);
    b.advance(1999);check(b.frame==2&&!b.playing);
    b.advance(1);check(b.playing&&!b.interval_active&&b.frame==2);
    b.advance(42);check(b.frame==3);
    for(int f:{57,112,167}){b.enter_frame(f);check(!b.playing&&b.remaining_ms==2000);b.play_callback();check(b.playing&&!b.interval_active);}
    b.enter_frame(220);b.frame_credit=0;b.advance(42);check(b.frame==1);
}
