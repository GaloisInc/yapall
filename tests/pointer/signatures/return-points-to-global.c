#include "assert.h"

extern void *return_points_to_global(void);

char g;

int main() {
  void *p = return_points_to_global();
  assert_may_alias(p, &g);
}
