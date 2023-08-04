#include "assert.h"

int main() {
  int u;
  int v;
  assert_may_alias(&u, &v);
  return 0;
}
