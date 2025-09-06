pub mod insert;
pub mod interval;

#[macro_export]
macro_rules! insert_patterns {
    ($graph:ident,
        $(
            $name:ident => [
                $([$($pat:expr),*]),*$(,)?
            ]
        ),*$(,)?
    ) => {

        $(
            let $name = $graph.insert_patterns([$(vec![$($pat),*]),*]);
        )*
    };
    ($graph:ident,
        $(
            ($name:ident, $idname:ident) => [
                $([$($pat:expr),*]),*$(,)?
            ]
        ),*$(,)?
    ) => {

        $(
            let ($name, $idname) = $graph.graph_mut().insert_patterns_with_ids([$(vec![$($pat),*]),*]);
        )*
    };
    ($graph:ident,
        $(
            ($name:ident, $idname:ident) =>
                [$($pat:expr),*]
        ),*$(,)?
    ) => {

        $(
            let ($name, $idname) = $graph.graph_mut().insert_pattern_with_id([$($pat),*]);
            let $idname = $idname.unwrap();
        )*
    };
}
