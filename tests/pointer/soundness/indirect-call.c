// RUN: clang -o - -emit-llvm -S -O1 %s 2>&1 | FileCheck %s

#include <stdio.h>

#include "assert.h"

void foo(void) __attribute__((noinline));
void bar(void) __attribute__((noinline));
void foo(void) {
  printf("foo\n");
  // CHECK: @assert
  assert_reachable();
}
void bar(void) {
  printf("bar\n");
  // CHECK: @assert
  assert_reachable();
}

int main(int argc, char *argv[]) {
  void (*func)(void);
  // CHECK: icmp
  // CHECK: select
  if (argc % 2 == 0) {
    func = foo;
  } else {
    func = bar;
  }
  // CHECK: call
  func();
  return 0;
}
