mod unit_test;
mod semantic;

use pest_derive::Parser;
use semantic::build_dir_func;

#[derive(Parser)]
#[grammar = "compiler_rules.pest"]
pub struct CSVParser;

fn main() {
    let programa = "
        programa miPrograma;
        vars
            x, y : entero;
            z : flotante;
        
        flotante function1(id1:entero, sincero:flotante, hola:entero){
            vars
                var1, x,y : flotante;
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

    match build_dir_func(programa) {
        Ok(dir) => {
            println!("\n── Directorio de funciones ──");
            for (nombre, entry) in &dir.funciones {
                println!("  {} : {}", nombre, entry.tipo);
                for (var, v) in &entry.vars {
                    println!("    {} : {}", var, v.tipo);
                }
            }
        }
        Err(e) => println!("{}", e),
    }
}