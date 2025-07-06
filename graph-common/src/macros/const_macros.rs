
#[macro_export]
macro_rules! const_map_result {
    (($expr:expr).map(|$inner:ident| $blk:block)) => {
        match $expr {
            Ok($inner) => Ok($blk),
            Err(e) => Err(e),
        }
    };

    (($expr:expr).map_err(|$inner:ident| $blk:block)) => {
        match $expr {
            Err($inner) => Err($blk),
            Ok(ok) => Ok(ok),
        }
    };

    (($expr:expr).map(|$inner_ok:ident| $blk_ok:block).map_err(|$inner_err:ident| $blk_err:block)) => {
        {
            let ok_map = const_map_result!(($expr).map(|$inner_ok| $blk_ok));
            const_map_result!((ok_map).map_err(|$inner_err| $blk_err))
        }
    };

    (($expr:expr).map(|_| $blk:block)) => {
        const_map_result!(($expr).map(|__| $blk))
    };

    (($expr:expr).map_err(|_| $blk:block)) => {
        const_map_result!(($expr).map_err(|__| $blk))
    };

    (($expr:expr).map(|_| $blk_ok:block).map_err(|$inner:ident| $blk_err:block)) => {
        const_map_result!(($expr).map(|__| $blk_ok:block).map_err(|$inner:ident| $blk_err:block))
    };

    (($expr:expr).map(|$inner:ident| $blk_ok:block).map_err(|_| $blk_err:block)) => {
        const_map_result!(($expr).map(|$inner| $blk_ok:block).map_err(|__| $blk_err:block))
    };

    (($expr:expr).map(|_| $blk_ok:block).map_err(|_| $blk_err:block)) => {
        const_map_result!(($expr).map(|__| $blk_ok:block).map_err(|$inner_err:ident| $blk_err:block))
    };
}

#[macro_export]
macro_rules! const_loop_range {
    {for $name:ident in ($initial:expr)..($end:expr).step($step:expr) $loop_block:block} => {
        {
            let mut __idx = $initial as isize;
            let __end = $end as isize;
            let __step = $step as isize;
            
            if __step > 0 {
                while __idx < __end {
                    {
                        let $name = __idx as usize;
                        $loop_block;
                    }
                    __idx = __idx.wrapping_add(__step);
                }
            } else {
                while __idx > __end {
                    {
                        let $name = __idx as usize;
                        $loop_block;
                    }
                    __idx = __idx.wrapping_add(__step);
                }
            }
        }
    };

    {for $name:ident in ($initial:expr)..($end:expr).step($step:expr) $loop_block:block} => {
        const_loop_range!{for $name in ($initial)..($end).step($step) $loop_block}
    };

    {for $name:ident in $initial:literal..($end:expr).step($step:expr) $loop_block:block} => {
        const_loop_range!{for $name in ($initial)..($end).step($step) $loop_block}
    };

    {for $name:ident in $initial:ident..($end:expr).step($step:expr) $loop_block:block} => {
        const_loop_range!{for $name in ($initial)..($end).step($step) $loop_block}
    };

    {for $name:ident in ($initial:expr)..$end:literal.step($step:expr) $loop_block:block} => {
        const_loop_range!{for $name in ($initial)..($end).step($step) $loop_block}
    };

    {for $name:ident in ($initial:expr)..$end:ident.step($step:expr) $loop_block:block} => {
        const_loop_range!{for $name in ($initial)..($end).step($step) $loop_block}
    };

    {for $name:ident in $initial:literal..$end:literal.step($step:expr) $loop_block:block} => {
        const_loop_range!{for $name in ($initial)..($end).step($step) $loop_block}
    };

    {for $name:ident in $initial:ident..$end:ident.step($step:expr) $loop_block:block} => {
        const_loop_range!{for $name in ($initial)..($end).step($step) $loop_block}
    };

    {for $name:ident in $initial:literal..$end:ident.step($step:expr) $loop_block:block} => {
        const_loop_range!{for $name in ($initial)..($end).step($step:expr) $loop_block}
    };

    {for $name:ident in $initial:ident..$end:literal.step($step:expr) $loop_block:block} => {
        const_loop_range!{for $name in ($initial)..($end).step($step:expr) $loop_block}
    };

    {for $name:ident in ($initial:expr)..=($end:expr).step($step:expr) $loop_block:block} => {
        let mut __idx = $initial as isize;
        let __end = $end as isize;
        let __step = $step as isize;

        const _: () = assert!(__step != 0, "step=0");
        
        if __step > 0 {
            while __idx <= __end {
                {
                    let $name = __idx;
                    $loop_block;
                }
                __idx = __idx.wrapping_add(__step);
            }
        } else {
            while __idx >= __end {
                {
                    let $name = __idx;
                    $loop_block;
                }
                __idx = __idx.wrapping_add(__step);
            }
        }
    };

    {for $name:ident in $initial:literal..=($end:expr).step($step:expr) $loop_block:block} => {
        const_loop_range!{for $name in ($initial)..=($end).step($step) $loop_block}
    };

    {for $name:ident in $initial:ident..=($end:expr).step($step:expr) $loop_block:block} => {
        const_loop_range!{for $name in ($initial)..=($end).step($step) $loop_block}
    };

    {for $name:ident in ($initial:expr)..=$end:literal.step($step:expr) $loop_block:block} => {
        const_loop_range!{for $name in ($initial)..=($end).step($step) $loop_block}
    };

    {for $name:ident in ($initial:expr)..=$end:ident.step($step:expr) $loop_block:block} => {
        const_loop_range!{for $name in ($initial)..=($end).step($step) $loop_block}
    };

    {for $name:ident in $initial:literal..=$end:literal.step($step:expr) $loop_block:block} => {
        const_loop_range!{for $name in ($initial)..=($end).step($step) $loop_block}
    };

    {for $name:ident in $initial:ident..=$end:ident.step($step:expr) $loop_block:block} => {
        const_loop_range!{for $name in ($initial)..=($end).step($step) $loop_block}
    };

    {for $name:ident in $initial:literal..=$end:ident.step($step:expr) $loop_block:block} => {
        const_loop_range!{for $name in ($initial)..=($end).step($step) $loop_block}
    };

    {for $name:ident in $initial:ident..=$end:literal.step($step:expr) $loop_block:block} => {
        const_loop_range!{for $name in ($initial)..=($end).step($step) $loop_block}
    };
}