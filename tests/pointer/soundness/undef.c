// RUN: clang -o - -emit-llvm -S -O1 %s 2>&1 | FileCheck %s
#include <stdlib.h>

int baz(int x) { return rand() % 2; }

// CHECK: main
int main(int argc, char const *argv[]) {
  // COM: TODO(#54)
  // COM: CHECK: i32 undef
  baz(argc);
  return 0;
}
