#include <cstdlib>
#include <memory>

#include "assert.h"

int main() {
  auto p = std::unique_ptr<char>((char *)malloc(1));
  assert_points_to_something(p.get());
  return 0;
}
