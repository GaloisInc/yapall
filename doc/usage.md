# Usage

First, compile a program to LLVM bitcode:

```sh
clang -emit-llvm -O1 -c -fno-discard-value-names tests/soundness/alloca.c
```

Then, run `yapall`:

```sh
yapall --signatures signatures.json alloca.bc
```
