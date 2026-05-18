use std::collections::HashMap;
use pest::Parser;
use crate::{CSVParser, Rule};

pub fn obtain_type(left_op: &str, right_op: &str, op: &str) -> &'static str {

    let type_index: HashMap<&str, usize> = HashMap::from([
        ("int",   0),
        ("float", 1),
    ]);

    let op_index: HashMap<&str, usize> = HashMap::from([
        ("+",  0),
        ("-",  1),
        ("*",  2),
        ("/",  3),
        (">",  4),
        ("<",  5),
        ("!=", 6),
        ("==", 7),
    ]);

    //        [left][right][op]
    let cube: [[[&str; 8]; 2]; 2] = [
        //  +        -        *        /        >       <       !=      ==
        // left = int
        [
        ["int",   "int",   "int",   "int",   "int", "int", "int", "int"],  // int
        ["float", "float", "float", "float", "int", "int", "int", "int"],  // float
        ],
        // left = float
        [
        ["float", "float", "float", "float", "int", "int", "int", "int"],  // int
        ["float", "float", "float", "float", "int", "int", "int", "int"],  // float
        ],
    ];

    let left_i  = match type_index.get(left_op)  { Some(&i) => i, None => return "err" };
    let right_i = match type_index.get(right_op) { Some(&i) => i, None => return "err" };
    let op_i    = match op_index.get(op) { Some(&i) => i, None => return "err" };

    return cube[left_i][right_i][op_i];
}

pub struct VarEntry {
    pub tipo: String,
}

pub struct FuncEntry {
    pub tipo: String,
    pub vars: HashMap<String, VarEntry>,
}

pub struct DirFunc {
    pub funciones: HashMap<String, FuncEntry>,
}

impl DirFunc {
    pub fn new() -> Self {
        DirFunc {
            funciones: HashMap::new(),
        }
    }

    // Agrega una función. Si ya existe, regresa error.
    pub fn agregar_funcion(&mut self, nombre: &str, tipo: &str) -> Result<(), String> {
        if self.funciones.contains_key(nombre) {
            return Err(format!("ERROR: funcion '{}' ya fue declarada", nombre));
        }
        self.funciones.insert(nombre.to_string(), FuncEntry {
            tipo: tipo.to_string(),
            vars: HashMap::new(),
        });
        Ok(())
    }

    // Agrega una variable dentro de una función. Si ya existe, regresa error.
    pub fn agregar_variable(&mut self, func: &str, nombre: &str, tipo: &str) -> Result<(), String> {
        let entry = self.funciones.get_mut(func).unwrap();
        if entry.vars.contains_key(nombre) {
            return Err(format!("ERROR: variable '{}' ya fue declarada en '{}'", nombre, func));
        }
        entry.vars.insert(nombre.to_string(), VarEntry {
            tipo: tipo.to_string(),
        });
        Ok(())
    }
}

// Función que camina el árbol de Pest
pub fn build_dir_func(source: &str) -> Result<DirFunc, String> {
    let pairs = CSVParser::parse(Rule::PROGRAM, source)
        .map_err(|e| format!("Error de sintaxis: {}", e))?;

    let mut dir = DirFunc::new();
    let mut current_func = String::new();  // en qué función estamos ahora
    let mut current_type = String::new();  // qué tipo vimos más reciente

    // Pest regresa un árbol. flatten() lo convierte en una lista plana de tokens
    // para poder recorrerlos uno por uno en orden.
    for pair in pairs.flatten() {
        match pair.as_rule() {

            // ── Punto neurálgico 1: nombre del programa ──────────────────
            // PROGRAM = { "programa" ~ ID ~ ... }
            // Cuando encontramos el ID del programa, creamos "global"
            Rule::PROGRAM => {
                let nombre = pair.into_inner()
                    .find(|p| p.as_rule() == Rule::ID)
                    .unwrap()
                    .as_str();

                dir.agregar_funcion("global", "void")?;
                current_func = "global".to_string();
                println!("✓ Programa '{}' registrado", nombre);
            }

            // ── Punto neurálgico 2: tipo encontrado ──────────────────────
            // Cada vez que vemos TYPE guardamos cuál es para usarlo después
            Rule::TYPE => {
                current_type = pair.as_str().to_string();
            }

            // ── Punto neurálgico 3: declaración de variables ─────────────
            // MULT_VARS = { ID ~ ("," ~ ID)* ~ ":" ~ TYPE ~ ";" }
            // Aquí pueden venir varios IDs con el mismo tipo
            Rule::MULT_VARS => {
                let mut ids: Vec<&str> = Vec::new();
                let mut tipo = String::new();

                for inner in pair.into_inner() {
                    match inner.as_rule() {
                        Rule::ID   => ids.push(inner.as_str()),
                        Rule::TYPE => tipo = inner.as_str().to_string(),
                        _ => {}
                    }
                }

                for id in ids {
                    dir.agregar_variable(&current_func, id, &tipo)?;
                    println!("  ✓ var '{}' : {} en '{}'", id, tipo, current_func);
                }
            }

            // ── Punto neurálgico 4: declaración de función ───────────────
            // FUNCS = { ("nula" | TYPE) ~ ID ~ "(" ~ ... }
            Rule::FUNCS => {
                let mut inner = pair.into_inner();

                // El primer token es el tipo de retorno ("nula" o TYPE)
                let tipo_ret = inner.next().unwrap().as_str().to_string();

                // El segundo token es el ID (nombre de la función)
                let nombre_func = inner.next().unwrap().as_str().to_string();

                dir.agregar_funcion(&nombre_func, &tipo_ret)?;
                current_func = nombre_func.clone();
                println!("✓ Funcion '{}' : {} registrada", nombre_func, tipo_ret);

                // Ahora procesamos los parámetros y variables locales
                // que vienen dentro de FUNCS
                for sub in inner {
                    match sub.as_rule() {
                        // Parámetros: SINGLE_VARS = { ID ~ ":" ~ TYPE }
                        Rule::SINGLE_VARS => {
                            let mut sv = sub.into_inner();
                            let id   = sv.next().unwrap().as_str();
                            let tipo = sv.next().unwrap().as_str();
                            dir.agregar_variable(&current_func, id, tipo)?;
                            println!("  ✓ param '{}' : {} en '{}'", id, tipo, current_func);
                        }
                        // Variables locales dentro de la función
                        Rule::MULT_VARS => {
                            let mut ids: Vec<&str> = Vec::new();
                            let mut tipo = String::new();
                            for inner2 in sub.into_inner() {
                                match inner2.as_rule() {
                                    Rule::ID   => ids.push(inner2.as_str()),
                                    Rule::TYPE => tipo = inner2.as_str().to_string(),
                                    _ => {}
                                }
                            }
                            for id in ids {
                                dir.agregar_variable(&current_func, id, &tipo)?;
                                println!("  ✓ var local '{}' : {} en '{}'", id, tipo, current_func);
                            }
                        }
                        _ => {}
                    }
                }

                // Al terminar la función regresamos a global
                current_func = "global".to_string();
            }

            _ => {}
        }
    }

    Ok(dir)
}