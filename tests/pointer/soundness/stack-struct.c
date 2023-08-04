// RUN: clang -o - -emit-llvm -S -O1 %s 2>&1 | FileCheck %s
#include "assert.h"

int main() {
  // CHECK: alloca
  int u;
  // CHECK: alloca
  int v;
  // CHECK: alloca
  struct {
    int *x;
    int *y;
  } pt = {&u, &v};
  // CHECK: getelementptr
  // CHECK: call {{.+}} @assert
  assert_points_to_something(&pt);
  // CHECK: call {{.+}} @assert
  assert_points_to_something(&pt.x);
  // CHECK: call {{.+}} @assert
  assert_points_to_something(&pt.y);
  return 0;
}
