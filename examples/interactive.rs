use std::ops::ControlFlow;

use cbit::cbit;

fn main() {
    assert_eq!(demo(&[1, 2, 3]), 6);
    assert_eq!(demo(&[1, 2, 3, 4, 101, 8]), -1);
}

fn demo(list: &[i32]) -> i32 {
    cbit!(for (accum, value) in reduce(0, list) {
        if *value > 100 {
            break -1;
        }
        accum + value
    })
}

fn reduce<T, I: IntoIterator, B>(
    initial: T,
    values: I,
    mut f: impl FnMut((T, I::Item)) -> ControlFlow<B, T>,
) -> ControlFlow<B, T> {
    let mut accum = initial;
    for value in values {
        accum = f((accum, value))?;
    }
    ControlFlow::Continue(accum)
}
