#pragma once
#include "presskit/gallery.hpp"
namespace presskit::enhanced {
// Opt-in correction: do not advance the displayed index while an image is loading.
std::optional<ImageRequest> next_when_ready(Gallery& gallery);
}
