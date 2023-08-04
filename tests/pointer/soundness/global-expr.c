// RUN: clang -o - -emit-llvm -S -O1 %s 2>&1 | FileCheck %s

#include "assert.h"

int g;
// CHECK: getelementptr {{.+}} @g
int *g_end = &g + 1;

int main() {
  // CHECK: call {{.+}} @assert
  assert_points_to_something(g_end);
  return 0;
}
