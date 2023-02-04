An implementation of the tree-walk Lox interpreter from Crafting Interpreters.

Notes
=====

While in general I've tried to follow the author's design as closely as
possible, since the original code is in Java and mine is in Rust, I've made
some changes to make my version more idiomatic.

The first significant change is a tweak to how expressions are traversed. The
author uses a visitor pattern, with expression subclasses providing an `accept`
method parameterized on the return type of the visitor. This is unnecessary and
even clunky in Rust, as it mixes dynamic dispatch with monomorphization, a
combination which tends lead to object safety violations. Instead, I use a more
functional version of the pattern, with visitors dispatching on expression
types via pattern matching. This seems reasonable enough, as the visitor
pattern is often used with the intention of achieving exactly this in languages
which lack pattern matching.
