mod grammar_unit_test;
mod compiler;
mod directory;
mod constants;
mod quadruples;
mod obj_writer;
mod virtual_machine;

use compiler::Compiler;
use obj_writer::ObjWriter;
use virtual_machine::VirtualMachine;

fn main() {
    let program = "
            programa fibrec;
            vars
                r : entero;
            entero fibonacci(n : entero) {
                vars res : entero;
                {
                    si (n < 2) {
                        res = n;
                    } sino {
                        res = fibonacci(n - 1) + fibonacci(n - 2);
                    };
                }
                return res;
            };
            inicio {
                r = fibonacci(10);
                escribe('fib', r);
            }
            fin
        ";
    let mut compiler = Compiler::new();
    if compiler.compile_program(program).is_ok() {
        ObjWriter::new(&compiler).write_to_file("program.obj").unwrap();

        // Load the obj file back into the Virtual Machine and run it.
        let mut vm = VirtualMachine::new();
        vm.execute("program.obj").unwrap();
    }
}

