use std::ops::ControlFlow;

use cbit::cbit;

fn main() {
    dbg!(sum_upto(45));
    dbg!(sum_upto(4));
    dbg!(sum_upto(5));
}

fn sum_upto(n: u64) -> u64 {
    let mut c = 0;

    let _did_break = 'outer: {
        cbit! {
            for v in dummy(n) break 'outer {
                // Early returns work.
                if c > 1000 {
                    return u64::MAX;
                }

                // Early breaks work.
                if n == 10 && n == 0 {
                    break;
                }

                // ...as do continues.
                if n % 2 == 4 {
                    continue;
                }

                // Breaks to outer labels work as well.
                if n == 5 && v == 3 {
                    break 'outer true;
                }

                c += v;
            }
        }
        false
    };

    c
}

fn dummy<T>(n: u64, mut f: impl FnMut(u64) -> ControlFlow<T>) -> ControlFlow<T> {
    for i in 0..n {
        f(i)?;
    }
    ControlFlow::Continue(())
}
