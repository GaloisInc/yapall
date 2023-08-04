// RUN: clang -o - -emit-llvm -S -O1 %s 2>&1 | FileCheck %s

#include <stdio.h>

#include "assert.h"

void __attribute__((noinline)) callee() {
  // CHECK: call {{.+}} @assert
  assert_reachable();
}

int main(int argc, char *argv[]) {
  // CHECK: call {{.+}} @callee
  callee();
}
