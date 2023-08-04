// RUN: clang -o - -emit-llvm -S -O1 %s 2>&1 | FileCheck %s

#include "assert.h"

int main() {
  int c;
  // CHECK: bitcast
  // CHECK: call {{.+}} @assert
  assert_points_to_something((void *)&c);
  return 0;
}
