// RUN: clang -o - -emit-llvm -S -O1 %s 2>&1 | FileCheck %s

#include <string.h>

#include "assert.h"

void __attribute__((noinline)) do_memcpy(void *dst, const void *src, size_t n) {
  // CHECK: call {{.+}} @llvm.memcpy
  memcpy(dst, src, n);
}

int main() {
  char c;
  char *a[32];
  for (int i = 0; i < 32; i++) {
    a[i] = &c;
  }
  char *b;
  do_memcpy(&b, &a, sizeof(char *));
  // CHECK: call {{.+}} @assert
  assert_points_to_something(b, a);
  return 0;
}
