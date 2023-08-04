// RUN: clang -o - -emit-llvm -S -O1 %s 2>&1 | FileCheck %s

#include <stdio.h>
#include <stdlib.h>

#include "assert.h"

int main() {
  // CHECK: alloca i8
  char c = 0;
  // CHECK: call {{.+}} @assert
  assert_points_to_something(&c);
}
