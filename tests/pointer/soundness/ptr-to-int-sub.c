// RUN: clang -o - -emit-llvm -S -O1 %s 2>&1 | FileCheck %s

#include <stdint.h>
#include <stdio.h>

#include "assert.h"

uintptr_t __attribute__((noinline)) sub1(uintptr_t p) {
  // COM: This results in a sub at O0
  // CHECK: add
  // CHECK: ret
  return p - 1;
}

int main(int argc, char *argv[]) {
  char c;
  // CHECK: alloca
  // CHECK: call {{.+}} @sub1
  // CHECK: call {{.+}} @assert
  assert_points_to_something((void *)sub1((uintptr_t)(&c + 1)));
}
