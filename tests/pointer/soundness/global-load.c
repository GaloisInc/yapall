// RUN: clang -o - -emit-llvm -S -O1 %s 2>&1 | FileCheck %s

#include "assert.h"

// CHECK: @g = {{.+}} 5
volatile char g = 5;
// CHECK: @gp = {{.+}} @g
volatile char *gp = &g;

int main() {
  // CHECK: load {{.+}} @gp
  // CHECK: call {{.+}} @assert
  assert_points_to_something((void *)gp);
  return 0;
}
