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
    let op_i    = match op_index.get(op)          { Some(&i) => i, None => return "err" };

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

// Procesa un bloque VARS y agrega todas sus variables a la función indicada
fn procesar_vars(dir: &mut DirFunc, vars_pair: pest::iterators::Pair<Rule>, func: &str) -> Result<(), String> {
    for mult in vars_pair.into_inner() {
        if mult.as_rule() == Rule::MULT_VARS {
            let mut ids  = Vec::new();
            let mut tipo = String::new();

            for t in mult.into_inner() {
                match t.as_rule() {
                    Rule::ID   => ids.push(t.as_str().to_string()),
                    Rule::TYPE => tipo = t.as_str().to_string(),
                    _ => {}
                }
            }

            for id in &ids {
                dir.agregar_variable(func, id, &tipo)?;
                // println!("  ✓ var '{}' : {} en '{}'", id, tipo, func);
            }
        }
    }
    Ok(())
}

// Función que camina el árbol de Pest
pub fn build_dir_func(source: &str) -> Result<DirFunc, String> {
    let pairs = CSVParser::parse(Rule::PROGRAM, source)
        .map_err(|e| format!("Error de sintaxis: {}", e))?;

    let mut dir = DirFunc::new();

    // El árbol tiene un solo nodo raíz: PROGRAM
    // Entramos manualmente con into_inner()
    // y así tener control exacto de en qué contexto estamos
    let program = pairs.into_iter().next().unwrap();

    for node in program.into_inner() {
        match node.as_rule() {

            // Punto 1: nombre del programa
            // PROGRAM = { "programa" ~ ID ~ ";" ~ VARS? ~ FUNCS* ~ "inicio" ~ BODY ~ "fin" }
            // Cuando encontramos el ID creamos la entrada "global" en DirFunc
            Rule::ID => {
                let nombre = node.as_str();
                dir.agregar_funcion("global", "void")?;
                // println!("✓ Programa '{}' registrado", nombre);
            }

            // Punto 2: variables globales
            // VARS = { "vars" ~ MULT_VARS+ }
            // Las procesamos y las metemos en "global"
            Rule::VARS => {
                procesar_vars(&mut dir, node, "global")?;
            }

            // Punto 3: declaración de función
            // FUNCS = { ("nula" | TYPE) ~ ID ~ "(" ~ SINGLE_VARS? ~ ")" ~ "{" ~ VARS? ~ BODY ~ "}" ~ ";" }
            Rule::FUNCS => {
                let mut inner = node.into_inner();

                // El primer token es el tipo de retorno ("nula" o TYPE)
                let tipo_ret    = inner.next().unwrap().as_str().to_string();

                // El segundo token es el ID (nombre de la función)
                let nombre_func = inner.next().unwrap().as_str().to_string();

                dir.agregar_funcion(&nombre_func, &tipo_ret)?;
                // println!("✓ Funcion '{}' : {} registrada", nombre_func, tipo_ret);

                // Procesamos el resto de los hijos de FUNCS
                for sub in inner {
                    match sub.as_rule() {

                        // Punto 4: parámetros de la función
                        // SINGLE_VARS = { ID ~ ":" ~ TYPE }
                        Rule::SINGLE_VARS => {
                            let mut sv  = sub.into_inner();
                            let id      = sv.next().unwrap().as_str();
                            let tipo    = sv.next().unwrap().as_str();
                            dir.agregar_variable(&nombre_func, id, tipo)?;
                            // println!("  ✓ param '{}' : {} en '{}'", id, tipo, nombre_func);
                        }

                        // Punto 5: variables locales de la función
                        // VARS = { "vars" ~ MULT_VARS+ }
                        Rule::VARS => {
                            procesar_vars(&mut dir, sub, &nombre_func)?;
                        }

                        // BODY y otros nodos los ignoramos
                        _ => {}
                    }
                }
            }

            // EOI y otros nodos los ignoramos
            _ => {}
        }
    }

    Ok(dir)
}