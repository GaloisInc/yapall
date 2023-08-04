// RUN: clang -o - -emit-llvm -S -O1 %s 2>&1 | FileCheck %s

#include "assert.h"

int g;

int main() {
  // CHECK: bitcast
  assert_points_to_something(&g);
  return 0;
}
