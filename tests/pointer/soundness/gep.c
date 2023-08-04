// RUN: clang -o - -emit-llvm -S -O1 %s 2>&1 | FileCheck %s

#include <stdio.h>
#include <stdlib.h>

#include "assert.h"

int main(int argc, char *argv[]) {
  // CHECK: call {{.+}} @malloc
  char *p = malloc(4);
  // CHECK: getelementptr
  // CHECK: call {{.+}} @assert
  assert_points_to_something(&p[2]);
}
