#include "assert.h"

extern void *return_alloc(void);

int main() {
  void *p = return_alloc();
  void *q = return_alloc();
  assert_disjoint(p, q);
}
