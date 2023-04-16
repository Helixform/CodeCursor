#[macro_export]
macro_rules! closure {
    (|$($arg:ident : $arg_type:ty),*| { $($body:tt)* }) => {
        $crate::closure!(new @ $($arg : $arg_type),* { $($body)* })
    };

    (|| { $($body:tt)* }) => {
        $crate::closure!(new @ { $($body)* })
    };

    ($create_fn:ident @ $($arg:ident : $arg_type:ty),* { $($body:tt)* }) => {
        wasm_bindgen::closure::Closure::$create_fn(move |$($arg : $arg_type),*| {
            $($body)*
        }) as wasm_bindgen::closure::Closure<dyn FnMut($($arg_type),*) -> _>
    };
}

#[macro_export]
macro_rules! closure_once {
    (|$($arg:ident : $arg_type:ty),*| { $($body:tt)* }) => {
        $crate::closure!(once @ $($arg : $arg_type),* { $($body)* })
    };

    (|| { $($body:tt)* }) => {
        $crate::closure!(once @ { $($body)* })
    };
}
