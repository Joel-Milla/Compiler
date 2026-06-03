mod grammar_unit_test;
mod semantic;
mod dir_func;
mod constants;

use semantic::Compiler;

fn main() {
    let advanced_programa = "
        programa miPrograma;
        vars
            x, y : entero;
            z : flotante;
        
        flotante function1(id1:entero, sincero:flotante, hola:entero){
            vars
                var1, x, y : flotante;
            {
                asignacion = 2 + 4;
                mientras(true) haz {escribe('hola');};
            }
        };
        inicio
            {
                x = 5;
                y = x + 3;
                si (x > y) {
                    x = x + 1;
                } sino {
                    y = y + 1;
                };
                mientras (x < 10) haz {
                    x = x + 1;
                };
                escribe ('resultado:', x);
            }
        fin
    ";

    let programa_basico = "
        programa miPrograma;
        vars
            x : entero;
            x : flotante;
        inicio
            {
                x = 5;
            }
        fin
    ";
    let mut compiler = Compiler::new();
    match compiler.compile_program(programa_basico) {
        Ok(_)  => println!("Compiled successfully"),
        Err(e) => println!("Error: {:?}", e),
    };
}

