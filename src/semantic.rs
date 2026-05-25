use std::collections::HashMap;
use pest::Parser;
use crate::{CSVParser, Rule};

// ─────────────────────────────────────────────────────────────────────────────
//  SEMANTIC CUBE
// ─────────────────────────────────────────────────────────────────────────────
pub fn obtain_type(left_op: &str, right_op: &str, op: &str) -> &'static str {
    let type_index: HashMap<&str, usize> = HashMap::from([
        ("entero",   0),
        ("flotante", 1),
    ]);
    let op_index: HashMap<&str, usize> = HashMap::from([
        ("+",  0), ("-",  1), ("*",  2), ("/",  3),
        (">",  4), ("<",  5), ("!=", 6), ("==", 7),
    ]);
    //        [left][right][op]
    let cube: [[[&str; 8]; 2]; 2] = [
        // left = entero
        [
            // right = entero
            ["entero",   "entero",   "entero",   "entero",   "entero", "entero", "entero", "entero"],
            // right = flotante
            ["flotante", "flotante", "flotante", "flotante", "entero", "entero", "entero", "entero"],
        ],
        // left = flotante
        [
            // right = entero
            ["flotante", "flotante", "flotante", "flotante", "entero", "entero", "entero", "entero"],
            // right = flotante
            ["flotante", "flotante", "flotante", "flotante", "entero", "entero", "entero", "entero"],
        ],
    ];
    let left_i  = match type_index.get(left_op)  { Some(&i) => i, None => return "err" };
    let right_i = match type_index.get(right_op) { Some(&i) => i, None => return "err" };
    let op_i    = match op_index.get(op)          { Some(&i) => i, None => return "err" };
    cube[left_i][right_i][op_i]
}

// ─────────────────────────────────────────────────────────────────────────────
//  SYMBOL TABLE STRUCTURES  (unchanged from your original)
// ─────────────────────────────────────────────────────────────────────────────
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
        DirFunc { funciones: HashMap::new() }
    }
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
    pub fn agregar_variable(&mut self, func: &str, nombre: &str, tipo: &str) -> Result<(), String> {
        let entry = self.funciones.get_mut(func).unwrap();
        if entry.vars.contains_key(nombre) {
            return Err(format!("ERROR: variable '{}' ya fue declarada en '{}'", nombre, func));
        }
        entry.vars.insert(nombre.to_string(), VarEntry { tipo: tipo.to_string() });
        Ok(())
    }
    // Look up the type of a variable, searching current function then global
    pub fn tipo_de(&self, func: &str, nombre: &str) -> Option<&str> {
        if let Some(entry) = self.funciones.get(func) {
            if let Some(v) = entry.vars.get(nombre) {
                return Some(&v.tipo);
            }
        }
        // fall back to global scope
        if let Some(entry) = self.funciones.get("global") {
            if let Some(v) = entry.vars.get(nombre) {
                return Some(&v.tipo);
            }
        }
        None
    }
}

// ─────────────────────────────────────────────────────────────────────────────
//  QUADRUPLE
// ─────────────────────────────────────────────────────────────────────────────
#[derive(Debug, Clone)]
pub struct Quad {
    pub op:     String,
    pub left:   String,
    pub right:  String,
    pub result: String,
}

impl Quad {
    fn new(op: &str, left: &str, right: &str, result: &str) -> Self {
        Quad {
            op:     op.to_string(),
            left:   left.to_string(),
            right:  right.to_string(),
            result: result.to_string(),
        }
    }
}

impl std::fmt::Display for Quad {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({:<4}, {:<10}, {:<10}, {})", self.op, self.left, self.right, self.result)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
//  COMPILER STATE  — holds the three stacks + the quadruple queue
// ─────────────────────────────────────────────────────────────────────────────
pub struct Compiler {
    pub dir:        DirFunc,
    pub quads:      Vec<Quad>,      // fila de cuádruplos
    poper:          Vec<String>,    // operator stack
    pila_o:         Vec<String>,    // operand stack
    p_types:        Vec<String>,    // type stack
    temp_count:     usize,
    pub errors:     Vec<String>,
    current_func:   String,         // scope we are currently inside
}

impl Compiler {
    pub fn new(dir: DirFunc) -> Self {
        Compiler {
            dir,
            quads:        Vec::new(),
            poper:        Vec::new(),
            pila_o:       Vec::new(),
            p_types:      Vec::new(),
            temp_count:   0,
            errors:       Vec::new(),
            current_func: "global".to_string(),
        }
    }

    // ── helpers ───────────────────────────────────────────────────────────────

    fn new_temp(&mut self) -> String {
        self.temp_count += 1;
        format!("t{}", self.temp_count)
    }

    fn emit(&mut self, op: &str, left: &str, right: &str, result: &str) {
        self.quads.push(Quad::new(op, left, right, result));
    }

    fn type_of(&self, name: &str) -> String {
        // numeric literal?
        if name.contains('.') { return "flotante".to_string(); }
        if name.chars().next().map(|c| c.is_ascii_digit()).unwrap_or(false) {
            return "entero".to_string();
        }
        // look up in symbol table
        match self.dir.tipo_de(&self.current_func, name) {
            Some(t) => t.to_string(),
            None    => "entero".to_string(),   // temporals are registered below
        }
    }

    // ── neuralgic-point actions ───────────────────────────────────────────────

    /// A1 – push operand + its type
    fn a1_push_operand(&mut self, name: &str) {
        let t = self.type_of(name);
        self.pila_o.push(name.to_string());
        self.p_types.push(t);
    }

    /// A2 – push + or - (resolving same-priority operators first)
    fn a2_push_add_sub(&mut self, op: &str) {
        self.resolve_if_top_in(&["+", "-"]);
        self.poper.push(op.to_string());
    }

    /// A3 – push * or / (resolving same-priority operators first)
    fn a3_push_mult_div(&mut self, op: &str) {
        self.resolve_if_top_in(&["*", "/"]);
        self.poper.push(op.to_string());
    }

    /// A4 – push relational operator (flush arithmetic first)
    fn a4_push_relational(&mut self, op: &str) {
        self.flush_arithmetic();
        self.resolve_if_top_in(&[">", "<", "!=", "=="]);
        self.poper.push(op.to_string());
    }

    /// A5 – open parenthesis: push fake bottom
    fn a5_open_paren(&mut self) {
        self.poper.push("(".to_string());
    }

    /// A6 – close parenthesis: resolve until matching '('
    fn a6_close_paren(&mut self) {
        while self.poper.last().map(|s| s.as_str()) != Some("(") && !self.poper.is_empty() {
            self.resolve_top();
        }
        self.poper.pop(); // discard '('
    }

    /// A7 – assignment statement
    fn a7_assignment(&mut self, var_name: &str) {
        if self.pila_o.is_empty() {
            self.errors.push(format!("Assignment error: empty stack for '{}'", var_name));
            return;
        }
        let rhs_val  = self.pila_o.pop().unwrap();
        let rhs_type = self.p_types.pop().unwrap();
        let lhs_type = self.type_of(var_name);
        let res_type = obtain_type(&lhs_type, &rhs_type, "=");
        // "=" is not in the cube, so we do a manual compatibility check instead
        if lhs_type != rhs_type && !(lhs_type == "flotante" && rhs_type == "entero") {
            self.errors.push(format!(
                "Type mismatch: cannot assign {} to {} ('{}')", rhs_type, lhs_type, var_name
            ));
            return;
        }
        self.emit("=", &rhs_val, "_", var_name);
    }

    /// A8 – escribe (print)
    fn a8_write(&mut self) {
        if self.pila_o.is_empty() { return; }
        let dato = self.pila_o.pop().unwrap();
        self.p_types.pop();
        self.emit("escribe", &dato, "_", "_");
    }

    /// A9 – string literal inside escribe
    fn a9_write_literal(&mut self, s: &str) {
        self.emit("escribe", s, "_", "_");
    }

    // ── internal resolution ───────────────────────────────────────────────────

    fn resolve_if_top_in(&mut self, ops: &[&str]) {
        if let Some(top) = self.poper.last() {
            if ops.contains(&top.as_str()) {
                self.resolve_top();
            }
        }
    }

    fn flush_arithmetic(&mut self) {
        while let Some(top) = self.poper.last() {
            if ["+", "-", "*", "/"].contains(&top.as_str()) {
                self.resolve_top();
            } else {
                break;
            }
        }
    }

    pub fn flush_all(&mut self) {
        while let Some(top) = self.poper.last() {
            if top == "(" { break; }
            self.resolve_top();
        }
    }

    fn resolve_top(&mut self) {
        if self.pila_o.len() < 2 || self.poper.is_empty() { return; }
        let right_val  = self.pila_o.pop().unwrap();
        let right_type = self.p_types.pop().unwrap();
        let left_val   = self.pila_o.pop().unwrap();
        let left_type  = self.p_types.pop().unwrap();
        let operator   = self.poper.pop().unwrap();

        let res_type = obtain_type(&left_type, &right_type, &operator);
        if res_type == "err" {
            self.errors.push(format!(
                "Type mismatch: {} {} {}", left_type, operator, right_type
            ));
            return;
        }

        let temp = self.new_temp();
        // register the temp in the current function's var table so type_of() can find it
        if let Some(entry) = self.dir.funciones.get_mut(&self.current_func) {
            entry.vars.insert(temp.clone(), VarEntry { tipo: res_type.to_string() });
        }

        self.emit(&operator, &left_val, &right_val, &temp);
        self.pila_o.push(temp);
        self.p_types.push(res_type.to_string());
    }

    // ─────────────────────────────────────────────────────────────────────────
    //  TREE WALKERS — one per grammar rule that affects expressions / statements
    // ─────────────────────────────────────────────────────────────────────────

    /// EXPRESION = { EXPR ~ (( ">" | "<" | "!=" | "==" ) ~ EXPR)? }
    pub fn walk_expresion(&mut self, pair: pest::iterators::Pair<Rule>) {
        let mut inner = pair.into_inner();
        // first EXPR
        if let Some(expr) = inner.next() {
            self.walk_expr(expr);
        }
        // optional relational operator + second EXPR
        while let Some(token) = inner.next() {
            match token.as_rule() {
                Rule::EXPR => self.walk_expr(token),
                // the operator is a raw string token — it comes between the two EXPRs
                _ => { self.a4_push_relational(token.as_str()); }
            }
        }
        self.flush_all();
    }

    /// EXPR = { TERM ~ (("+" | "-") ~ TERM)* }
    fn walk_expr(&mut self, pair: pest::iterators::Pair<Rule>) {
        let mut pending_op: Option<String> = None;
        for token in pair.into_inner() {
            match token.as_rule() {
                Rule::TERM => {
                    if let Some(op) = pending_op.take() {
                        self.a2_push_add_sub(&op);
                    }
                    self.walk_term(token);
                }
                _ => {
                    // "+" or "-" literal
                    pending_op = Some(token.as_str().to_string());
                }
            }
        }
    }

    /// TERM = { FACTOR ~ (("*" | "/") ~ FACTOR)* }
    fn walk_term(&mut self, pair: pest::iterators::Pair<Rule>) {
        let mut pending_op: Option<String> = None;
        for token in pair.into_inner() {
            match token.as_rule() {
                Rule::FACTOR => {
                    if let Some(op) = pending_op.take() {
                        self.a3_push_mult_div(&op);
                    }
                    self.walk_factor(token);
                }
                _ => {
                    pending_op = Some(token.as_str().to_string());
                }
            }
        }
    }

    /// FACTOR = { SINGLE_EXPR | CALL | SINGLE_FACTOR }
    fn walk_factor(&mut self, pair: pest::iterators::Pair<Rule>) {
        let inner = pair.into_inner().next().unwrap();
        match inner.as_rule() {
            Rule::SINGLE_EXPR   => self.walk_single_expr(inner),
            Rule::CALL          => self.walk_call(inner),
            Rule::SINGLE_FACTOR => self.walk_single_factor(inner),
            _                   => {}
        }
    }

    /// SINGLE_EXPR = { "(" ~ EXPRESION ~ ")" }
    fn walk_single_expr(&mut self, pair: pest::iterators::Pair<Rule>) {
        self.a5_open_paren();
        for inner in pair.into_inner() {
            if inner.as_rule() == Rule::EXPRESION {
                self.walk_expresion(inner);
            }
        }
        self.a6_close_paren();
    }

    /// SINGLE_FACTOR = { ("+" | "-")? ~ (ID | CTE) }
    fn walk_single_factor(&mut self, pair: pest::iterators::Pair<Rule>) {
        let mut unary: Option<String> = None;
        for token in pair.into_inner() {
            match token.as_rule() {
                Rule::ID  => {
                    let name = token.as_str();
                    if let Some(u) = unary.take() {
                        // emit  0 - name  or  0 + name
                        self.a1_push_operand("0");
                        self.a1_push_operand(name);
                        if u == "-" { self.a2_push_add_sub("-"); }
                        // unary + is a no-op
                        self.resolve_if_top_in(&["+", "-"]);
                    } else {
                        self.a1_push_operand(name);
                    }
                }
                Rule::CTE => {
                    let val = token.as_str();
                    self.a1_push_operand(val);
                    if let Some(u) = unary.take() {
                        if u == "-" {
                            // negate: pop the constant, push -val
                            self.pila_o.pop();
                            let neg = format!("-{}", val);
                            let t   = self.p_types.pop().unwrap();
                            self.pila_o.push(neg);
                            self.p_types.push(t);
                        }
                    }
                }
                _ => {
                    // "+" or "-" unary sign
                    unary = Some(token.as_str().to_string());
                }
            }
        }
    }

    /// CALL = { ID ~ "(" ~ (EXPRESION ~ ("," ~ EXPRESION)*)? ~ ")" }
    fn walk_call(&mut self, pair: pest::iterators::Pair<Rule>) {
        let mut inner  = pair.into_inner();
        let func_name  = inner.next().unwrap().as_str().to_string();
        let mut params = Vec::new();

        for node in inner {
            if node.as_rule() == Rule::EXPRESION {
                self.walk_expresion(node);
                // the result of each param expression ends up on top of PilaO
                if let (Some(val), Some(t)) = (self.pila_o.pop(), self.p_types.pop()) {
                    params.push((val, t));
                }
            }
        }

        // emit PARAM quads
        for (val, _) in &params {
            self.emit("param", val, "_", "_");
        }

        // emit GOSUB
        let temp = self.new_temp();
        let ret_type = self.dir.funciones.get(&func_name)
            .map(|f| f.tipo.clone())
            .unwrap_or_else(|| "entero".to_string());

        if let Some(entry) = self.dir.funciones.get_mut(&self.current_func) {
            entry.vars.insert(temp.clone(), VarEntry { tipo: ret_type.clone() });
        }

        self.emit("gosub", &func_name, "_", &temp);
        self.pila_o.push(temp);
        self.p_types.push(ret_type);
    }

    // ── statement walkers ─────────────────────────────────────────────────────

    /// ASIGNA = { ID ~ "=" ~ EXPRESION ~ ";" }
    pub fn walk_asigna(&mut self, pair: pest::iterators::Pair<Rule>) {
        let mut inner = pair.into_inner();
        let var_name  = inner.next().unwrap().as_str().to_string();
        let expr      = inner.next().unwrap();
        self.walk_expresion(expr);
        self.a7_assignment(&var_name);
    }

    /// IMPRIME = { "escribe" ~ "(" ~ PRINT_STATEMENT ~ ("," ~ PRINT_STATEMENT)* ~ ")" ~ ";" }
    pub fn walk_imprime(&mut self, pair: pest::iterators::Pair<Rule>) {
        for ps in pair.into_inner() {
            // ps is PRINT_STATEMENT = { EXPRESION | LETRERO }
            if ps.as_rule() == Rule::PRINT_STATEMENT {
                let inner = ps.into_inner().next().unwrap();
                match inner.as_rule() {
                    Rule::EXPRESION => {
                        self.walk_expresion(inner);
                        self.a8_write();
                    }
                    Rule::LETRERO => {
                        self.a9_write_literal(inner.as_str());
                    }
                    _ => {}
                }
            }
        }
    }

    /// CONDICION = { "si" ~ "(" ~ EXPRESION ~ ")" ~ BODY ~ ("sino" ~ BODY)? ~ ";" }
    pub fn walk_condicion(&mut self, pair: pest::iterators::Pair<Rule>) {
        let mut inner = pair.into_inner();

        // evaluate condition expression
        let expr = inner.next().unwrap();
        self.walk_expresion(expr);

        // pop condition result, emit GotoF
        let cond = self.pila_o.pop().unwrap_or("_".to_string());
        self.p_types.pop();
        let gotof_idx = self.quads.len();
        self.emit("GotoF", &cond, "_", "?"); // "?" = patch later

        // walk the true BODY
        let true_body = inner.next().unwrap();
        self.walk_body(true_body);

        // check for optional "sino" BODY
        if let Some(sino_body) = inner.next() {
            // emit Goto to skip the else branch
            let goto_idx = self.quads.len();
            self.emit("Goto", "_", "_", "?");

            // patch the GotoF to jump here (start of else)
            let else_start = self.quads.len();
            self.quads[gotof_idx].result = else_start.to_string();

            self.walk_body(sino_body);

            // patch the Goto to jump past the else
            let after_else = self.quads.len();
            self.quads[goto_idx].result = after_else.to_string();
        } else {
            // no else — patch GotoF to jump past the true body
            let after_if = self.quads.len();
            self.quads[gotof_idx].result = after_if.to_string();
        }
    }

    /// CICLO = { "mientras" ~ "(" ~ EXPRESION ~ ")" ~ "haz" ~ BODY ~ ";" }
    pub fn walk_ciclo(&mut self, pair: pest::iterators::Pair<Rule>) {
        let loop_start = self.quads.len(); // remember where the condition starts
        let mut inner  = pair.into_inner();

        // evaluate condition
        let expr = inner.next().unwrap();
        self.walk_expresion(expr);

        let cond = self.pila_o.pop().unwrap_or("_".to_string());
        self.p_types.pop();
        let gotof_idx = self.quads.len();
        self.emit("GotoF", &cond, "_", "?");

        // walk body
        let body = inner.next().unwrap();
        self.walk_body(body);

        // jump back to loop start
        self.emit("Goto", "_", "_", &loop_start.to_string());

        // patch GotoF to exit
        let after_loop = self.quads.len();
        self.quads[gotof_idx].result = after_loop.to_string();
    }

    /// BODY = { "{" ~ ESTATUTO* ~ "}" }
    pub fn walk_body(&mut self, pair: pest::iterators::Pair<Rule>) {
        for estatuto in pair.into_inner() {
            if estatuto.as_rule() == Rule::ESTATUTO {
                self.walk_estatuto(estatuto);
            }
        }
    }

    /// ESTATUTO = { ASIGNA | CONDICION | CICLO | (CALL ~ ";") | IMPRIME | ("[" ~ ESTATUTO* ~ "]") }
    pub fn walk_estatuto(&mut self, pair: pest::iterators::Pair<Rule>) {
        let inner = pair.into_inner().next().unwrap();
        match inner.as_rule() {
            Rule::ASIGNA    => self.walk_asigna(inner),
            Rule::CONDICION => self.walk_condicion(inner),
            Rule::CICLO     => self.walk_ciclo(inner),
            Rule::CALL      => self.walk_call(inner),
            Rule::IMPRIME   => self.walk_imprime(inner),
            // bracketed block: [ ESTATUTO* ]
            _ => {
                for s in inner.into_inner() {
                    if s.as_rule() == Rule::ESTATUTO {
                        self.walk_estatuto(s);
                    }
                }
            }
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
//  helper: process VARS block into DirFunc  (unchanged from your original)
// ─────────────────────────────────────────────────────────────────────────────
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
            }
        }
    }
    Ok(())
}

// ─────────────────────────────────────────────────────────────────────────────
//  PUBLIC ENTRY POINT
// ─────────────────────────────────────────────────────────────────────────────
pub fn build_dir_func(source: &str) -> Result<DirFunc, String> {
    let pairs = CSVParser::parse(Rule::PROGRAM, source)
        .map_err(|e| format!("Error de sintaxis: {}", e))?;

    let mut dir = DirFunc::new();
    let program = pairs.into_iter().next().unwrap();

    for node in program.into_inner() {
        match node.as_rule() {
            Rule::ID => {
                dir.agregar_funcion("global", "void")?;
            }
            Rule::VARS => {
                procesar_vars(&mut dir, node, "global")?;
            }
            Rule::FUNCS => {
                let mut inner       = node.into_inner();
                let tipo_ret        = inner.next().unwrap().as_str().to_string();
                let nombre_func     = inner.next().unwrap().as_str().to_string();
                dir.agregar_funcion(&nombre_func, &tipo_ret)?;
                for sub in inner {
                    match sub.as_rule() {
                        Rule::SINGLE_VARS => {
                            let mut sv = sub.into_inner();
                            let id     = sv.next().unwrap().as_str();
                            let tipo   = sv.next().unwrap().as_str();
                            dir.agregar_variable(&nombre_func, id, tipo)?;
                        }
                        Rule::VARS => {
                            procesar_vars(&mut dir, sub, &nombre_func)?;
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }

    Ok(dir)
}

// ─────────────────────────────────────────────────────────────────────────────
//  NEW: build_compiler  — builds DirFunc AND generates all quadruples
// ─────────────────────────────────────────────────────────────────────────────
pub fn build_compiler(source: &str) -> Result<Compiler, String> {
    // ── pass 1: build the symbol table (same as before) ──────────────────────
    let dir = build_dir_func(source)?;

    // ── pass 2: generate quadruples ──────────────────────────────────────────
    let pairs = CSVParser::parse(Rule::PROGRAM, source)
        .map_err(|e| format!("Error de sintaxis: {}", e))?;

    let mut compiler = Compiler::new(dir);
    let program      = pairs.into_iter().next().unwrap();

    for node in program.into_inner() {
        match node.as_rule() {

            // walk each function body with its scope
            Rule::FUNCS => {
                let mut inner   = node.into_inner();
                let _tipo_ret   = inner.next().unwrap(); // skip return type
                let nombre_func = inner.next().unwrap().as_str().to_string();
                compiler.current_func = nombre_func.clone();

                for sub in inner {
                    if sub.as_rule() == Rule::BODY {
                        compiler.walk_body(sub);
                    }
                }
                compiler.current_func = "global".to_string();
            }

            // walk the main body
            Rule::BODY => {
                compiler.current_func = "global".to_string();
                compiler.walk_body(node);
            }

            _ => {}
        }
    }

    if !compiler.errors.is_empty() {
        return Err(compiler.errors.join("\n"));
    }

    Ok(compiler)
}