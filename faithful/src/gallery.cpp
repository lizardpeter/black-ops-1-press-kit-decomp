#include "presskit/gallery.hpp"
#include <limits>
#include <stdexcept>
namespace presskit {
// presskit 3143802: AVM1 Add2, separator, Add2, total, Add2.
std::string picture_num(int zero_based_index, int total) {
    return std::to_string(static_cast<long long>(zero_based_index) + 1) + " / " + std::to_string(total);
}
ImageRequest Gallery::request() const {
    return {index, 1, 1, picture_num(index,total)};
}
// presskit 3143432: increment happens before testing loaded == filesize.
std::optional<ImageRequest> Gallery::next() {
    if (static_cast<long long>(index) >= static_cast<long long>(total)-1) return std::nullopt;
    ++index;
    if (loaded != filesize) return std::nullopt;
    return request();
}
// presskit 3143575: decrement if scr > 0; no loading guard.
std::optional<ImageRequest> Gallery::previous() {
    if (index <= 0) return std::nullopt;
    --index;
    return request();
}
// presskit 3143696: does not reset scr, tests load equality only.
std::optional<ImageRequest> Gallery::first() const {
    if (loaded != filesize) return std::nullopt;
    return request();
}
// Frame 101: scrn_1..scrn_20; play timeline then set zero-based scr.
bool Gallery::select_thumbnail(int button) {
    if(button<1 || button>20) return false;
    timeline_playing=true;
    index=button-1;
    return true;
}
// Frame 105 loadXML at 3143190. Host XML parser supplies first child's text
// from each entry, in source order. Original clears descriptions but never fills it.
std::optional<ImageRequest> Gallery::xml_loaded(bool success, const std::vector<std::string>& first_child_values) {
    if(!success) { content="file not loaded!"; return std::nullopt; }
    if(first_child_values.size()>static_cast<std::size_t>(std::numeric_limits<int>::max())) throw std::length_error("XML count exceeds native state");
    images=first_child_values;
    descriptions.clear();
    total=static_cast<int>(images.size());
    return first();
}
// 3144803: left and right Flash key codes, no wrap.
std::optional<ImageRequest> Gallery::key_down(int code) {
    if(code==37)return previous();
    if(code==39)return next();
    return std::nullopt;
}
// 3144962, 3145004.
std::optional<ImageRequest> Gallery::previous_release(){return previous();}
std::optional<ImageRequest> Gallery::next_release(){return next();}
}
