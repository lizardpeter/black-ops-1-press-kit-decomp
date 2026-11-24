#include "presskit/enhanced.hpp"
#include <cstdlib>
int main(){presskit::Gallery g;g.total=3;g.loaded=1;g.filesize=2;if(presskit::enhanced::next_when_ready(g)||g.index!=0)std::abort();g.loaded=2;if(!presskit::enhanced::next_when_ready(g)||g.index!=1)std::abort();}
