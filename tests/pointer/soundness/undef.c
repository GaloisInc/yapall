// RUN: clang -o - -emit-llvm -S -O1 %s 2>&1 | FileCheck %s
#include <stdlib.h>

int baz(int x) { return rand() % 2; }

int main(int argc, char const *argv[]) {
  // COM: I don't really know why this works.
  // CHECK: i32 undef
  baz(argc);
  return 0;
}
