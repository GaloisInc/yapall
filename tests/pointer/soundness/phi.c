// RUN: clang -o - -emit-llvm -S -O1 %s 2>&1 | FileCheck %s
#include "stdlib.h"

#include "assert.h"

int main() {
  char c;
  void *p = &c;
  for (int i = 0; i < 32; i++) {
    // CHECK: phi
    // CHECK: call {{.+}} @assert
    assert_points_to_something(p);
    p = malloc(1);
  }
  return 0;
}
