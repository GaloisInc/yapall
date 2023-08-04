// RUN: clang -o - -emit-llvm -S -O1 %s 2>&1 | FileCheck %s
#include "assert.h"

// CHECK: @u
int u;
// CHECK: @v
int v;
// CHECK: @pt = {{.+}}@u{{.+}}@v
struct {
  int *x;
  int *y;
} pt = {&u, &v};

int main() {
  // CHECK: call {{.+}} @assert
  assert_points_to_something(&pt);
  // CHECK: call {{.+}} @assert
  assert_points_to_something(&pt.x);
  // CHECK: call {{.+}} @assert{{.+}} getelementptr
  assert_points_to_something(&pt.y);
  return 0;
}
