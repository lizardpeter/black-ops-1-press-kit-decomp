#include "presskit/background.hpp"
#include <cmath>
#include <stdexcept>
namespace presskit {
// Four identical authored Play definitions, SHA-256 8939b815...:
// 2377744, 2378254, 2378764, 2379274.
// clearInterval(nInterval); play();
void Background::play_callback(){
    interval_active=false;remaining_ms=0;playing=true;
}
// Original enclosing frame actions: stop(); nInterval=setInterval(Play,2000).
void Background::enter_frame(int f){
    frame=f;
    if(f==2||f==57||f==112||f==167){
        playing=false;interval_active=true;remaining_ms=pause.count();frame_credit=0;
    }
}
void Background::advance(double elapsed_ms){
    if(!std::isfinite(elapsed_ms)||elapsed_ms<0)throw std::invalid_argument("invalid elapsed time");
    constexpr double period=1000.0/fps;
    while(elapsed_ms>0){
        if(interval_active){
            if(elapsed_ms<remaining_ms){remaining_ms-=elapsed_ms;return;}
            elapsed_ms-=remaining_ms;play_callback();
        }else if(playing){
            const double needed=period-frame_credit;
            if(elapsed_ms<needed){frame_credit+=elapsed_ms;return;}
            elapsed_ms-=needed;frame_credit=0;
            enter_frame(frame==last_frame?1:frame+1);
        }else return;
    }
}
}
