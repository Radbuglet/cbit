use std::ops::ControlFlow;

use cbit::cbit;

fn main() {
    dbg!(sum_upto(45));
    dbg!(sum_upto(4));
    dbg!(sum_upto(5));
}

fn sum_upto(n: u64) -> u64 {
    let mut c = 0;
    cbit! {
        for v in dummy(n) {
            c += v;

            if c > 1000 {
                return u64::MAX;
            }

            if n == 5 && v == 3 {
                break;
            }
        }
    }
    c
}

fn dummy<T>(n: u64, mut f: impl FnMut(u64) -> ControlFlow<T>) -> ControlFlow<T> {
    for i in 0..n {
        f(i)?;
    }
    ControlFlow::Continue(())
}
