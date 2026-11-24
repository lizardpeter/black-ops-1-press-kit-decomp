#include "presskit/gallery.hpp"
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
}
