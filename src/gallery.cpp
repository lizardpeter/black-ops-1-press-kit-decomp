#include "presskit/gallery.hpp"
namespace presskit {
// presskit 3143802: AVM1 Add2, separator, Add2, total, Add2.
std::string picture_num(int zero_based_index, int total) {
    return std::to_string(static_cast<long long>(zero_based_index) + 1) + " / " + std::to_string(total);
}
}
