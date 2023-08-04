// RUN: clang -o - -emit-llvm -S -O1 %s 2>&1 | FileCheck %s

#include "assert.h"

// CHECK: @g
char g;

int main() {
  // CHECK: call {{.+}} @assert
  assert_points_to_something(&g);
  return 0;
}
