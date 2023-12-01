use std::ops::ControlFlow;

fn up_to<B>(n: u64, mut f: impl FnMut(u64) -> ControlFlow<B>) -> ControlFlow<B> {
    for i in 0..n {
        f(i)?;
    }
    ControlFlow::Continue(())
}

fn demo(n: u64) -> u64 {
    let mut c = 0;
    'outer_1: loop {
        let something = 'outer_2: {
            cbit::cbit!(for i in up_to(n) break loop 'outer_1, 'outer_2 {
                if i == 5 && c < 20 {
                    continue 'outer_1;
                }
                if i == 8 {
                    break 'outer_2 c < 10;
                }
                c += i;
            });
            false
        };

        if something {
            assert!(c < 10);
        } else {
            break;
        }
    }
    c
}

fn main() {
    demo(10);
}
