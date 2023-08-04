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

[ascent]: https://github.com/s-arash/ascent

## Bibliography

To understand Yapall and how it works, you may want to review:

- The [pointer analysis tutorial][tutorial] of Smaragdakis and Balatsouras

- Bravenboer, M. and Smaragdakis, Y., 2009, October. Strictly declarative
  specification of sophisticated points-to analyses. In Proceedings of the 24th
  ACM SIGPLAN conference on Object oriented programming systems languages and
  applications (pp. 243-262).

- Balatsouras, G. and Smaragdakis, Y., 2016, September. Structure-sensitive
  points-to analysis for C and C++. In International Static Analysis Symposium
  (pp. 84-104). Springer, Berlin, Heidelberg.

[tutorial]: http://yanniss.github.io/points-to-tutorial15.pdf
