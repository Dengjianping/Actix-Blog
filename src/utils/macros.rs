#![macro_use]
#[macro_export]
// mod macros {
    // create a struct by macro
macro_rules! new_struct {
    // struct like this
    // struct Foo {name: String, year: u32}
    ($struct_name: ident, $vis: ident, [$($deri: meta), *], ($($field: ident => $type: ty), +)) => {
        #[derive(
            $(
                $deri,
            )*
        )]
        $vis struct $struct_name {
            $(
                $vis $field: $type,
            )+
        }
    };

    // struct with no field name like this
    // struct Foo(String, u32)
    ($struct_name: ident, [$($deri: meta), *], ($($type: ty), +)) => {
        #[derive(
            $($deri,)*
            )
        ]
        struct $struct_name(
            $(
                $type,
            )+
        );
    };
}
// }