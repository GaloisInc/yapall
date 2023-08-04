#include <stdlib.h>

#include "assert.h"

int main() {
  char *x = malloc(1);
  assert_disjoint(x, x);
  return 0;
}
