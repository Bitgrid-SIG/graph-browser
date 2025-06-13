#[macro_export]
macro_rules! ss_const_panic {
    ($msg:literal,$ss_err:ident) => {
        match $ss_err {
            SSErrorType::StringTooBig => panic!(core::concat!($msg, "StringTooBig")),
            SSErrorType::StringEmpty => panic!(core::concat!($msg, "StringEmpty")),
            SSErrorType::MatchNotFound => panic!(core::concat!($msg, "MatchNotFound")),
            SSErrorType::Uninit => panic!(core::concat!($msg, "Uninitialized")),
            SSErrorType::Utf8Error(_) => panic!(core::concat!($msg, "UTF-8 Error")),
        }
    };
}

#[macro_export]
macro_rules! const_loop_range {
    ($lower:expr ; $name:ident < $upper:expr ; $loop_block:block) => {
        let mut $name = $lower;
        while $name < $upper {
            $loop_block;
            $name += 1;
        }
    };

    ($lower:expr ; $name:ident <= $upper:expr ; $loop_block:block) => {
        let mut $name = $lower;
        while $name <= $upper {
            $loop_block;
            $name += 1;
        }
    };
}

#[macro_export]
macro_rules! const_unwrap {
    ([$var:ident -> $at:ty]) => {
        if let Ok(value) = $var {
            let inner: $at = value;
            inner
        } else {
            panic!("Tried to unwrap none or error!");
        }
    };

    ([$expr:expr => $at:ty]) => {
        {
            let val = $expr;
            const_unwrap!([val -> $at])
        }
    };
}

#[macro_export]
macro_rules! const_map_result {
    ([let $inner:ident = ($outer:ident -> Result<$at:ty>)]: $impl:block) => {
        if let Ok($inner) = $outer {
            let result: $at = $impl;
            Ok(result)
        } else if let Err(e) = $outer {
            Err(e)
        } else {
            unreachable!()
        }
    };

    ([(let $inner:ident = $expr:expr) => Result<$at:ty>]: $impl:block) => {
        {
            let $inner = $expr;
            const_map_result!([let $inner = ($inner -> Result<$at>)]: $impl)
        }
    };
}

#[macro_export]
macro_rules! const_map_collection {
    ($var:ident.map($name:ident: [$bt:ty; $bl:literal]) -> [$at:ty; $al:literal] $impl:block) => {
        {
            const_assert_eq!($bl, $al);

            let _inputs = $var;
            let mut results: [$at; $al] = unsafe { core::mem::zeroed() };

            const_loop_range!(0; i < $bl ; {
                let $name = unsafe { core::ptr::read(&_inputs[i]) };
                results[i] = $impl;
            });

            results
        }
    };

    ($var:ident.map($name:ident: [$bt:ty;$bl:ident]) -> [$at:ty;$al:ident] $impl:block) => {
        {
            let _inputs: [$bt; $bl] = $var;
            let mut results: [$at; $al] = unsafe { core::mem::zeroed() };

            const_loop_range!(0; i < $bl ; {
                let $name = unsafe { core::ptr::read(&_inputs[i]) };
                results[i] = $impl;
            });

            results
        }
    };
}

#[macro_export]
macro_rules! const_unwrap_result {
    ($e:expr) => {
        if let Ok(k) = $e {
            k
        } else {
            panic!("Tried to unwrap an error!")
        }
    };
}

// #[macro_export]
// macro_rules! const_include_lines {
//     ($($filename:literal)+) => {

//     };
// }
