// RUN: clang++ -o - -emit-llvm -S -O1 %s 2>&1 | FileCheck %s

#include <stdexcept>

#include "assert.h"

int main() {
  // CHECK: call {{.+}} @__cxa_allocate_exception
  try {
    // CHECK: invoke {{.+}} @__cxa_throw
    throw std::logic_error("hello, exceptions!");
  } catch (const std::logic_error &e) {
    // CHECK: landingpad
    // CHECK: @__cxa_begin_catch
    // CHECK: invoke {{.+}} @assert
    assert_points_to_something((void *)&e);
    // CHECK: @__cxa_end_catch
  }
  return 0;
}
