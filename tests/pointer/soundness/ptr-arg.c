// RUN: clang -o - -emit-llvm -S -O1 %s 2>&1 | FileCheck %s

#include <stdio.h>
#include <stdlib.h>

#include "assert.h"

void __attribute__((noinline)) callee(char *p) {
  // CHECK: call {{.+}} @assert
  assert_points_to_something(p);
}

int main(int argc, char *argv[]) {
  // CHECK: call {{.+}} @malloc
  char *c = malloc(1);
  // CHECK: call {{.+}} @callee
  callee(c);
}
