mod unit_test;

use pest::Parser;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "compiler_rules.pest"]
pub struct CSVParser;

fn main() {
    let successful_parse = CSVParser::parse(Rule::PROGRAM, 
    "programa patito; 
            vars 
                x : entero;
                y : flotante;
                x,y,a30, entero_23 : entero;
                prueba: flotante;
            "
        );
    println!("{:?}", successful_parse);
}

/*
    use std::collections::VecDeque;
    use std::collections::BTreeMap;
    // Se implementa una estructura stack con las funciones principales
    struct Stack {
        stack: Vec<u32>,
    }
    
    impl Stack {
        fn new() -> Self {
            // Se usa un vector para implementar el stack
            Stack { stack: Vec::new() }
    }
    
    fn push(&mut self, number: u32) {
        self.stack.push(number);
    }
    
    fn pop(&mut self) {
        // pop() retorna un opcional. Se tiene que extraer de forma segura.
        let top = self.stack.pop();
        match top {
            Some(value) => println!("Removed number = {}", value),
            None => println!("No value"),
        }
    }
    
    fn top(&self) {
        // last() retorna un opcional. Se tiene que extraer de forma segura.
        let top = self.stack.last();
        match top {
            Some(value) => println!("Top number = {}", value),
            None => println!("No value"),
        }
    }
}

fn testingOfDataStructures() {
    // Se utiliza la estructura de datos stack creada
    println!("**Stack");
    let mut stack : Stack = Stack::new();
    stack.push(32);
    stack.push(2);
    // Se crea el stack = [32, 2]
    stack.pop(); // stack=[32]
    stack.top(); // top=32
    println!("-------------------");
    
    // Se usa la libreria de std para usar una queue
    println!("**Queue");
    let mut queue: VecDeque<u32> = VecDeque::new();
    queue.push_back(32);
    queue.push_back(64);
    let front = queue.pop_front();
    match front {
        Some(value) => println!("Removed number = {}", value),
        None => println!("No value"),
    }
    let front = queue.front();
    match front {
        Some(value) => println!("Number at the front = {}", value),
        None => println!("No value"),
    }
    println!("-------------------");
    
    // Se usa la libreria de BTreeMap para tener un mapa ordenado
    println!("**Map");
    let mut movies: BTreeMap<i32, &str> = BTreeMap::new();
    movies.insert(0, "Iron Man");
    movies.insert(1, "Captain America");
    movies.insert(2, "Black Panther");
    
    // Se busca un id especifico
    if !movies.contains_key(&4) {
        println!("Ghost Rider movie not found");
    }
    // Se elimina una pelicula
    movies.remove(&2);
    println!("Black panther movie removed");
    
    // Se buscan las peliculas
    let to_find = [1,2];
    for id_movie in &to_find {
        match movies.get(id_movie) {
            Some(movie_title) => println!("Title = {}", movie_title),
            None => println!("`{id_movie}` id_key not found.")
        }
    }
}
*/