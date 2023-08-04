#include <stdlib.h>

#include "assert.h"

extern void *arg_memcpy_arg(void *dst, void *src);

char g;

int main() {
  char c;
  char **src = malloc(sizeof(char *));
  char **dst = malloc(sizeof(char *));
  *src = &c;
  arg_memcpy_arg(dst, src);
  assert_may_alias(*dst, *src);
}
