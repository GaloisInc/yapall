// RUN: clang -o - -emit-llvm -S -O1 %s 2>&1 | FileCheck %s
#include "assert.h"

void __attribute__((noinline)) callee(uint64_t x) {
  // CHECK: call {{.+}} @assert
  assert_constant(x);
}

int main() {
  // CHECK: call {{.+}} @callee
  callee(0);
  return 0;
}
