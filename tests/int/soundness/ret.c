// RUN: clang -o - -emit-llvm -S -O1 %s 2>&1 | FileCheck %s
#include "assert.h"

uint64_t __attribute__((noinline)) ret0(void) {
  // CHECK: ret i64 0
  return 0;
}

int main() {
  // COM: LLVM really aggressively inlines ret0
  // COM: CHECK: call {{.+}} @ret0
  uint64_t val = ret0();
  assert_constant(val);
  // CHECK: ret i64 0
  return 0;
}
