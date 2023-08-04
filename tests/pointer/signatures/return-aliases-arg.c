#include <stdlib.h>

#include "assert.h"

extern void *return_aliases_arg(void *);

int main() {
  void *p = malloc(1);
  void *q = return_aliases_arg(p);
  assert_points_to_something(q);
}
