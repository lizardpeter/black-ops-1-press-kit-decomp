#include "presskit/enhanced.hpp"
namespace presskit::enhanced {
std::optional<ImageRequest> next_when_ready(Gallery& gallery) {
    if(gallery.loaded!=gallery.filesize) return std::nullopt;
    return gallery.next();
}
}
