// TODO: make this more DRY -- the expression types (Binary, Grouping, etc.)
// are repeated in the `ast_struct!` and `ast_enum!` expansions for example.

/// Given an identifier `expr` and a list of `ident, type` pairs, make:
/// - A struct for the given expression type `expr`.
/// - An impl for `S` with `new` and `make method. The `new` method takes the
///   list of `ident: type` pairs as parameters and returns the raw struct.
///   The `make` convenience method takes the same parameters and returns
///   a Box<Expr(S)>>.
#[macro_export]
macro_rules! ast_struct {
    ($enum_name: ident, $lifetime: lifetime, $struct_name: ident, $($field: ident, $type: ty),*) => {
        #[derive(Debug)]
        pub struct $struct_name<$lifetime> {
            $(
                pub $field: $type,
            )*
        }

        impl<$lifetime> $struct_name<$lifetime> {
            pub fn new($($field: $type,)*) -> Self {
                Self { $($field,)* }
            }

            #[allow(unused)]
            pub fn var($($field: $type,)*) -> $enum_name<'a> {
                $enum_name::$struct_name($struct_name::new($($field,)*))
            }

            #[allow(unused)]
            pub fn make($($field: $type,)*) -> Box<$enum_name<'a>> {
                Box::new($enum_name::$struct_name($struct_name::new($($field,)*)))
            }
        }
    };
}

#[macro_export]
macro_rules! ast_enum {
    ($enum_name: ident, $lifetime: lifetime, $($item: ident),*) => {
        #[derive(Debug)]
        pub enum $enum_name<$lifetime> {
            $(
                $item($item<$lifetime>),
            )*
        }
    };
}
