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
    let factorial_main = "
        programa fact_main;
        vars
            f, i : entero;
        inicio {
            f = 1;
            i = 1;
            mientras (i < 6) haz {
                f = f * i;
                i = i + 1;
            };
            escribe('factorial de 5:', f);
        }
        fin
    ";
    
    let factorial_func = "
        programa fact_func;
        vars
            resultado : entero;
        entero factorial(n : entero) {
            vars f, i : entero;
            {
                f = 1;
                i = 1;
                mientras (i < n) haz {
                    i = i + 1;
                    f = f * i;
                };
            }
            return f;
        };
        inicio {
            resultado = factorial(5);
            escribe('factorial de 5:', resultado);
        }
        fin
    ";
    
    let fibonacci_main = "
        programa fib_main;
        vars
            a, b, t, i : entero;
        inicio {
            a = 0;
            b = 1;
            i = 0;
            mientras (i < 10) haz {
                t = a + b;
                a = b;
                b = t;
                i = i + 1;
            };
            escribe('fibonacci 10:', a);
        }
        fin
    ";

    let fibonacci_func = "
        programa fib_func;
        vars
            salida : entero;
        entero fib_iter(n : entero) {
            vars a, b, t, i : entero;
            {
                a = 0;
                b = 1;
                i = 0;
                mientras (i < n) haz {
                    t = a + b;
                    a = b;
                    b = t;
                    i = i + 1;
                };
            }
            return a;
        };
        inicio {
            salida = fib_iter(10);
            escribe('fibonacci 10:', salida);
        }
        fin
    ";

    let fibonacci_double = "
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
            r = fibonacci(12);
            escribe('fib', r);
        }
        fin
    ";

    let example_prof = "
        programa pelos;
        vars
            i, j : entero;
        entero uno(x : entero) {
            {
            }
            return x * 2;
        };
        entero dos(x : entero) {
            {
            }
            return x * uno(x);
        };
        inicio {
            i = 5;
            escribe(dos(i + 3 - 1));
        }
        fin
    ";

    let mut compiler = Compiler::new();
    match compiler.compile_program(fibonacci_double) {
        Ok(()) => {
            ObjWriter::new(&compiler).write_to_file("program.obj").unwrap();
            // Load the obj file back into the Virtual Machine and run it.
            let mut vm = VirtualMachine::new();
            vm.execute("program.obj").unwrap();
        }
        Err(e) => println!("Compile error: {}", e),
    }
}

