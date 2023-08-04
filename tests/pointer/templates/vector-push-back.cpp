#include <cstdlib>
#include <vector>

#include "assert.h"

int main() {
  std::vector<char *> v;
  v.push_back((char *)malloc(1));
  assert_points_to_something(v.back());
  return 0;
}
