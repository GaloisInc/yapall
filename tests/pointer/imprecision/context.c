#include "assert.h"

char *__attribute__((noinline)) add1(char *p) { return p + 1; }

int main() {
  char u;
  char v;
  char *x = add1(&u);
  char *y = add1(&v);
  assert_may_alias(x, y);
  return 0;
}
