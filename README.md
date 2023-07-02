## lox-tw

An implementation of the tree-walk Lox interpreter from Crafting Interpreters.
The interpreter is feature complete and passes all of the tests available
in the [author's repo](https://github.com/munificent/craftinginterpreters).

Run Lox scripts with `cargo run $script`, or try out the REPL with `cargo run`.


### Notes

While in general I've tried to follow the author's design as closely as
possible, since the original code is in Java and mine is in Rust, I've made
some changes to make my version more idiomatic, or in some cases substitute
for Java features which Rust lacks.

The first significant change is a tweak to how expressions are traversed. The
author uses a visitor pattern, with expression subclasses providing an `accept`
method parameterized on the return type of the visitor. This is unnecessary and
even clunky in Rust, as it mixes dynamic dispatch with monomorphization, a
combination which tends to lead to object safety violations. Instead, I use a
more functional version of the pattern, with visitors dispatching on expression
types via pattern matching. This seems reasonable enough to me, as the visitor
pattern is often used with the intention of achieving exactly this in languages
which lack pattern matching.

Another substantial deviation is in how garbage collection is handled in the
interpreter. Whereas the author leans on the JVM's GC to do the interpreter's
GC as well, there is no such luxury in Rust. `Rc` and co. are insufficient,
since they don't collect reference cycles, and reference cycles are easily
created in Lox code. Instead, I'm using the Rust `Gc` crate, which implements a
mark and sweep garbage collector. Unfortunately, performance does suffer a bit
relative to the author's implementation as a result.

One more significant change, and somewhat related the previous one, is around
the Lox-to-host language type mapping. The author's implementation borrows the
Java object model: every object in Lox is implemented by its corresponding Java
object, and he makes frequent use of the fact that everything inherits from
`Object`. Since Rust lacks such an object model, I've created an `Object` enum
whose variants cover Lox's various primitive types.
