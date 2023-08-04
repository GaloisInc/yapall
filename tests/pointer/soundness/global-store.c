// RUN: clang -o - -emit-llvm -S -O1 %s 2>&1 | FileCheck %s

#include "assert.h"

volatile void *g;

int main() {
  // CHECK: alloca
  char c;
  // CHECK: store
  g = &c;
  // The following line prevents Clang from optimizing away the load from @g
  // for the next assertion, because it may change @g (since it's volatile).
  //
  // CHECK: call {{.+}} @assert
  assert_points_to_something(&c);
  // CHECK: load
  // CHECK: call {{.+}} @assert
  assert_points_to_something((void *)g);
  return 0;
}
