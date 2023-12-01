use std::ops::ControlFlow;

use cbit::cbit;

fn main() {
    dbg!(demo(45));
    dbg!(demo(4));
    dbg!(demo(5));
}

fn demo(n: u64) -> u64 {
    let mut c = 0;

    'even_more_outer: loop {
        let _did_break = 'outer: {
            cbit!('me: for v in up_to(n) break 'outer, loop 'even_more_outer {
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
                    continue 'me;
                }

                // Breaks to outer labels work as well...
                if n == 5 && v == 3 {
                    break 'outer true;
                }

                // ...as do continues.
                if c < 10 {
                    c += 1;
                    continue 'even_more_outer;
                }

                c += v;
            });
            false
        };

        break;
    }

    c
}

fn up_to<T>(n: u64, mut f: impl FnMut(u64) -> ControlFlow<T>) -> ControlFlow<T> {
    for i in 0..n {
        f(i)?;
    }
    ControlFlow::Continue(())
}
