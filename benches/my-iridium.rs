#[macro_use]
extern crate criterion;
extern crate my_iridium;

use criterion::Criterion;
use my_iridium::assembler::{PIE_HEADER_LENGTH, PIE_HEADER_PREFIX};
use my_iridium::vm::VM;

fn get_test_vm() -> VM {
    let mut vm = VM::new();
    vm.registers[0] = 5;
    vm.registers[1] = 10;
    vm
}

mod arithmetic {
    use super::*;criterion_group! {
        name = arithmetic;
        config = Criterion::default();
        targets = execute_add, execute_sub, execute_mul, execute_div,
    }

    fn execute_add(c: &mut Criterion) {
        let clos = || {
            let mut vm = get_test_vm();
            vm.program = vec![1, 0, 1, 2];
            vm.run_once();
        };

        c.bench_function(
            "execute_add",
            move |b| b.iter(clos),
        );
    }

    fn execute_sub(c: &mut Criterion) {
        let clos = || {
            let mut vm = get_test_vm();
            vm.program = vec![2, 1, 0, 2];
            vm.run_once();
        };

        c.bench_function("execute_sub", move |b| b.iter(clos));
    }

    fn execute_mul(c: &mut Criterion) {
        let clos = || {
            let mut vm = get_test_vm();
            vm.program = vec![3, 0, 1, 2];
            vm.run_once();
        };

        c.bench_function("execute_mul", move |b| b.iter(clos));
    }

    fn execute_div(c: &mut Criterion) {
        let clos = || {
            let mut vm = get_test_vm();
            vm.program = vec![4, 1, 0, 2];
            vm.run_once();
        };

        c.bench_function("execute_div", move |b| b.iter(clos));
    }
}


criterion_main!(arithmetic::arithmetic);