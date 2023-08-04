// RUN: clang -o - -emit-llvm -S -O1 %s 2>&1 | FileCheck %s

#include "assert.h"

// CHECK: @u
int u;
// CHECK: @v
int v;
// CHECK: @a = {{.+}}@u{{.+}}@v
int *a[2] = {&u, &v};

int main() {
  // CHECK: call {{.+}} @assert
  assert_points_to_something(&a);
  // CHECK: call {{.+}} @assert
  assert_points_to_something(&a[0]);
  // CHECK: call {{.+}} @assert{{.+}} getelementptr
  assert_points_to_something(&a[1]);
  return 0;
}
