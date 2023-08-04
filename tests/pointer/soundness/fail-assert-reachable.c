// RUN: clang -o - -emit-llvm -S -O1 %s 2>&1 | FileCheck %s

#include "assert.h"

void foo() {
  // CHECK: call {{.+}} @assert
  assert_reachable();
}

int main() { return 0; }
