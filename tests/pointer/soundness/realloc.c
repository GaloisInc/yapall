// RUN: clang -o - -emit-llvm -S -O1 %s 2>&1 | FileCheck %s

#include <stdlib.h>

#include "assert.h"

int main() {
  // CHECK: call {{.+}} @malloc
  char *c = malloc(1);
  // CHECK: call {{.+}} @realloc
  char *c2 = realloc(c, 4);
  // CHECK: call {{.+}} @assert
  assert_points_to_something(c2);
  return 0;
}
