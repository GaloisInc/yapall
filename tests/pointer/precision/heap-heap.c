#include <stdlib.h>

#include "assert.h"

int main() {
  char *x = malloc(1);
  char *y = malloc(1);
  assert_disjoint(x, y);
  return 0;
}
