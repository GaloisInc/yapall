// RUN: clang -o - -emit-llvm -S -O1 %s 2>&1 | FileCheck %s
#include "assert.h"

int main() {
  // CHECK: call {{.+}} @assert
  assert_constant(0);
  return 0;
}
