#include <cstdlib>
#include <stack>

#include "assert.h"

int main() {
  std::stack<char *> s;
  s.push((char *)malloc(1));
  assert_points_to_something(s.top());
  return 0;
}
