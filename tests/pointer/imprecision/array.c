#include "assert.h"

int main() {
  char u;
  char v;
  char a[2] = {u, v};
  assert_may_alias(&a[0], &a[1]);
}
