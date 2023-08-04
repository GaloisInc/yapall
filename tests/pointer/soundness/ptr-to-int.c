// RUN: clang -o - -emit-llvm -S -O1 %s 2>&1 | FileCheck %s

#include <stdint.h>
#include <stdio.h>

#include "assert.h"

uintptr_t __attribute__((noinline)) add1(uintptr_t p) {
  // CHECK: add
  // CHECK: ret
  return p + 1;
}

int main(int argc, char *argv[]) {
  char c;
  // CHECK: alloca
  // CHECK: call {{.+}} @add1
  // CHECK: call {{.+}} @assert
  assert_points_to_something((void *)add1((uintptr_t)&c));
}
