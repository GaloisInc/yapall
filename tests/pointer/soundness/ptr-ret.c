// RUN: clang -o - -emit-llvm -S -O1 %s 2>&1 | FileCheck %s

#include <stdio.h>
#include <stdlib.h>

#include "assert.h"

void *__attribute__((noinline)) add1(void *p) {
  // CHECK: add
  // CHECK: ret
  return p + 1;
}

int main(int argc, char *argv[]) {
  // CHECK: call {{.+}} @malloc
  char *c = malloc(1);
  // CHECK: call {{.+}} @add1
  void *p = add1(c);
  // CHECK: call {{.+}} @assert
  assert_points_to_something(p);
}
