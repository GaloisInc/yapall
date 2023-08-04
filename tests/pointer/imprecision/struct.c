#include "assert.h"

int main() {
  int u;
  int v;
  struct {
    int *x;
    int *y;
  } pt = {&u, &v};
  assert_may_alias(&pt.x, &pt.y);
}
