// RUN: clang -o - -emit-llvm -S -O1 %s 2>&1 | FileCheck %s

#include <errno.h>

#include "assert.h"

int main() {
  // CHECK: call {{.+}} @__errno_location
  // CHECK: call {{.+}} @assert
  assert_points_to_something(&errno);
}
