// TODO: make this more DRY -- the expression types (Binary, Grouping, etc.)
// are repeated in the `ast_struct!` and `ast_enum!` expansions for example.

/// Given an identifier `expr` and a list of `ident, type` pairs, define:
/// - A struct for the given expression type `expr`.
/// - An impl for `S` with `new` and `make methods. The `new` method takes the
///   list of `ident: type` pairs as parameters and returns the raw struct.
///   The `make` convenience method takes the same parameters and returns
///   an Expr(Gc<S>).
#[macro_export]
macro_rules! ast_struct {
    ($enum_name: ident, $struct_name: ident, $($field: ident, $type: ty),*) => {
        #[derive(Clone, Debug, Finalize, Trace)]
        pub struct $struct_name {
            $(
                pub $field: $type,
            )*
            id_: usize
        }

        impl $struct_name {
            pub fn new($($field: $type,)*) -> Self {
                Self { $($field,)* id_: unique_usize() }
            }

            pub fn make($($field: $type,)*) -> $enum_name {
                $enum_name::$struct_name(Gc::new($struct_name::new($($field,)*)))
            }

            #[allow(unused)]
            pub fn id(&self) -> usize {
                self.id_
            }
        }
    };
}

#[macro_export]
macro_rules! ast_enum {
    ($enum_name: ident, $($item: ident),*) => {
        #[derive(Clone, Debug, Finalize, Trace)]
        pub enum $enum_name {
            $(
                $item(Gc<$item>),
            )*
        }
    };
}
