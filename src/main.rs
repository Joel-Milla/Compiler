mod grammar_unit_test;
mod compiler;
mod directory;
mod constants;
mod quadruples;

use compiler::Compiler;

fn main() {
    let program = "";
    let mut compiler = Compiler::new();
    let result = compiler.compile_program(program);

    if result.is_ok() {
        
    }
}

