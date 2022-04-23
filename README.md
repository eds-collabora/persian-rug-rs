# persian-rug - it really ties the room together

This is a framework for building and mutating collections of objects
that refer to one another. It supports having multiple different types
participating in the graph. It does not require that links form
a directed acyclic graph, or tree, or anything convenient.

To do this, all objects are stored in a centralised holder (a
[Context](https://docs.rs/persian-rug/latest/persian_rug/struct.Context.html),
which passes out
[Proxy](https://docs.rs/persian-rug/latest/persian_rug/struct.Proxy.html)
objects to serve as pointers. All access requires the context to be
present, and this has some limitations with Rust's current mutable
reference rules (it's not possible to borrow the same context mutably
more than once, even when accessing different pieces of data within
it).

**Deletion of objects is not supported at present.** Contexts many
only grow to encompass more objects, old objects may not be
retired. This restriction might be lifted a future version of this
crate.

## License

This crate is made available under either an
[Apache-2.0](https://opensource.org/licenses/Apache-2.0) or an [MIT
license](https://opensource.org/licenses/MIT).
