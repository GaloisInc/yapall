// RUN: clang -o - -emit-llvm -S -O1 %s 2>&1 | FileCheck %s
// https://www.gnu.org/software/libc/manual/html_node/Variadic-Example.html#Variadic-Example
#include <stdarg.h>

#include "assert.h"

int add_em_up(int count, int *x, ...) {
  va_list ap;
  int i, sum;

  // CHECK: call void @llvm.va_start
  va_start(ap, x);

  sum = 0;
  for (i = 0; i < count; i++) {
    int *arg = va_arg(ap, int *);
    // CHECK: call {{.+}} @assert
    assert_points_to_something(arg);
    sum += *arg;
  }

  // CHECK: call void @llvm.va_end
  va_end(ap);
  return sum;
}

int main(void) {
  int x = 3;
  int y = 5;
  int z = 6;
  return add_em_up(3, &x, &y, &z);
}
