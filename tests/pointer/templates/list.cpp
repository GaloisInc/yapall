#include <cstdlib>
#include <list>

#include "assert.h"

int main() {
  std::list<char *> a{(char *)malloc(1), (char *)malloc(1)};
  assert_points_to_something(a.front());
  assert_points_to_something(a.back());
  return 0;
}
