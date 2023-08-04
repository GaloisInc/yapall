// RUN: clang -o - -emit-llvm -S -O1 %s 2>&1 | FileCheck %s
#include "assert.h"

int main(int argc, char *argv[]) {
  // CHECK: call {{.+}} @assert
  assert_points_to_something(argv[0]);
  return 0;
}
