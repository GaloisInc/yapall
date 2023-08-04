// RUN: clang -o - -emit-llvm -S -O1 %s 2>&1 | FileCheck %s

#include <stdlib.h>

#include "assert.h"

int main() {
  // CHECK: alloca i8
  char *c = alloca(1);
  // CHECK: call {{.+}} @assert
  assert_points_to_something(c);
  return 0;
}
