#pragma once
#include <string>
#include <optional>
namespace presskit {
std::string picture_num(int zero_based_index, int total);
struct ImageRequest { int index; int alpha; int load_argument; std::string counter; };
struct Gallery {
    int index=0;
    int total=0;
    double loaded=0, filesize=0;
    std::optional<ImageRequest> next();
    std::optional<ImageRequest> previous();
    std::optional<ImageRequest> first() const;
private:
    ImageRequest request() const;
};
}
