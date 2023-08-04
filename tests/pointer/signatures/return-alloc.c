#include "assert.h"

extern void *return_alloc(void);

int main() {
  void *p = return_alloc();
  assert_points_to_something(p);
}
