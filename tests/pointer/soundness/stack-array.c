// RUN: clang -o - -emit-llvm -S -O1 %s 2>&1 | FileCheck %s
#include "assert.h"

int main() {
  // CHECK: alloca
  int u;
  // CHECK: alloca
  int v;
  // CHECK: alloca
  int *a[2] = {&u, &v};
  // CHECK: getelementptr
  // CHECK: call {{.+}} @assert
  assert_points_to_something(&a);
  // CHECK: call {{.+}} @assert
  assert_points_to_something(&a[0]);
  // CHECK: call {{.+}} @assert
  assert_points_to_something(&a[1]);
  return 0;
}
