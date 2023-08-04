// RUN: clang -o - -emit-llvm -S -O1 %s 2>&1 | FileCheck %s
#include <stdio.h>

#include "assert.h"

// CHECK: @stderr = external

int main() {
  // CHECK: call {{.+}} @assert{{.+}}@stderr
  assert_points_to_something(&stderr);
}
