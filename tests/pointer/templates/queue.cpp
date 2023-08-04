#include <cstdlib>
#include <queue>

#include "assert.h"

int main() {
  std::queue<char *> s;
  s.push((char *)malloc(1));
  assert_points_to_something(s.front());
  assert_points_to_something(s.back());
  return 0;
}
