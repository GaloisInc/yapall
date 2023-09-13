# Yet Another Pointer Analysis for LLVM

Yapall is a precise and scalable pointer analysis for LLVM. The output of
Yapall can be used for a variety of program analysis tasks, including:

- Creation of callgraphs with precise handling of indirect function calls and
  virtual method calls
- Precise inter-procedural control- and data-flow analysis
- Answering may-alias queries

Yapall is k-callsite sensitive for configurable k, field-, array-, and 
flow-insensitive, and performs on-the-fly callgraph construction. Yapall is
written using [Ascent][ascent] and so is highly parallel.

For more information, see [the documentation](./doc).

[ascent]: https://github.com/s-arash/ascent

## Acknowledgments

This material is based upon work supported by the Defense Advanced Research
Projects Agency (DARPA) under Contract No. N66001-21-C-4023. Any opinions,
findings and conclusions or recommendations expressed in this material are
those of the author(s) and do not necessarily reflect the views of DARPA.

## Distribution

DISTRIBUTION STATEMENT A. Approved for public release: distribution unlimited.
