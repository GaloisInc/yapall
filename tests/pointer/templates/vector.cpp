#include <cstdlib>
#include <vector>

#include "assert.h"

int main() {
  std::vector<char *> v{(char *)malloc(1)};
  assert_points_to_something(v[0]);
  return 0;
}
