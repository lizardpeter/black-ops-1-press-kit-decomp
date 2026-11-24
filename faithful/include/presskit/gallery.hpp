#pragma once
#include <string>
#include <optional>
#include <vector>
namespace presskit {
std::string picture_num(int zero_based_index, int total);
struct ImageRequest { int index; int alpha; int load_argument; std::string counter; };
struct Gallery {
    int index=0;
    int total=0;
    double loaded=0, filesize=0;
    bool timeline_playing=false;
    std::vector<std::string> images, descriptions;
    std::string content;
    bool select_thumbnail(int button);
    std::optional<ImageRequest> key_down(int code);
    std::optional<ImageRequest> previous_release();
    std::optional<ImageRequest> next_release();
    std::optional<ImageRequest> xml_loaded(bool success, const std::vector<std::string>& first_child_values);
    std::optional<ImageRequest> next();
    std::optional<ImageRequest> previous();
    std::optional<ImageRequest> first() const;
private:
    ImageRequest request() const;
};
}
