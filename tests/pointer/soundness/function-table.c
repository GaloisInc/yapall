// RUN: clang -o - -emit-llvm -S -O1 %s 2>&1 | FileCheck %s

#include <stdio.h>

#include "assert.h"

void foo(void) __attribute__((noinline));
void bar(void) __attribute__((noinline));
void baz(void) __attribute__((noinline));

int main(int argc, char const *argv[]) {
  void (*table[])(void) = {
      foo,
      bar,
      baz,
  };
  // CHECK: getelementptr
  // CHECK: call
  table[argc % 3]();
  return 0;
}

void foo(void) {
  printf("foo\n");
  // CHECK: call {{.+}} @assert
  assert_reachable();
}

void bar(void) {
  printf("bar\n");
  // CHECK: call {{.+}} @assert
  assert_reachable();
}

void baz(void) {
  printf("baz\n");
  // CHECK: call {{.+}} @assert
  assert_reachable();
}
