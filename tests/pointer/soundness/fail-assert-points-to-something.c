// RUN: clang -o - -emit-llvm -S -O1 %s 2>&1 | FileCheck %s

#include "assert.h"
#include <stddef.h>

int main(int argc, char *argv[]) {
  // CHECK: call {{.+}} @assert
  assert_points_to_something((void *)(size_t)argc);
  return 0;
}
