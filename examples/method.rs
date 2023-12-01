use std::ops::ControlFlow;

use cbit::cbit;

fn main() {
    cbit!(for _ in Demo.method::<_>() {
        println!("We ran!");
    });
}

struct Demo;

impl Demo {
    pub fn method<B>(&self, f: impl FnOnce(()) -> ControlFlow<B>) -> ControlFlow<B> {
        f(())
    }
}
