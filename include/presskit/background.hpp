#pragma once
#include <chrono>
namespace presskit {
// BG sprite 75, frames 2/57/112/167. Independent of root playback.
struct Background {
    static constexpr int fps=24, last_frame=220;
    static constexpr auto pause=std::chrono::milliseconds(2000);
    int frame=1;
    bool playing=true, interval_active=false;
    double remaining_ms=0, frame_credit=0;
    void enter_frame(int f);
    void play_callback();
    void advance(double elapsed_ms);
};
}
