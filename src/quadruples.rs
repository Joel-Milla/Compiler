use crate::{constants::{ENTERO_TYPE, FLOTANTE_TYPE, ASSIGN, GOTOF, GOTO}};
use std::{collections::HashMap};

#[derive(Debug, PartialEq)]
pub struct Quad {
    operator : String,
    left : String,
    right : String,
    result : String
}

impl Quad {
    pub fn new(operator : String, left : String, right : String, result : String) -> Self {
        Quad {
            operator, left, right, result
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Quadruples {
    pub quads : Vec<Quad>, // Were all the quadruples are storesd
    stack_vars : Vec<(String, String)>, // (name, type) of the variable
    stack_op : Vec<String>, // Stack to save the operators
    stack_jump : Vec<usize>,
    pub temp_count : usize
}

impl Quadruples {
    pub fn new() -> Self {
        Quadruples {
            quads : Vec::new(),
            stack_vars : Vec::new(),
            stack_op : Vec::new(),
            stack_jump : Vec::new(),
            temp_count : 0,
        }
    }

    //* Next functions help generate quads */

    /// This function takes the top of the stack (left, right) operators, and its operator type.
    /// Get a new temporal variable, and then push it to the quadruple list
    pub fn resolve(&mut self) -> Result<(), String> {
        let right = self.stack_vars.pop().unwrap();
        let left = self.stack_vars.pop().unwrap();
        let op = self.stack_op.pop().unwrap();

        if op == ASSIGN {

            // Validate if both variables have the same type
            if right.1 != left.1 {
                return Err(format!("Type mismatch: cannot assign {} to {}", right.1, left.1));
            }

            // Add the quadruple of assigning operator
            self.quads.push(Quad::new(ASSIGN.to_string(), right.0, "".to_string(), left.0));
            return Ok(());
        }

        let temp = self.new_temp();
        let temp_type = Quadruples::obtain_type(&left.1, &right.1, &op)?;

        // Add the quad to the quadruple list, and save the temporal variable to the stack
        self.quads.push(Quad::new(op, left.0, right.0,temp.clone()));
        self.push_var(temp, temp_type);

        Ok(())
    }

    /// Generates a new temporal variable based on a counter
    pub fn new_temp(&mut self) -> String {
        self.temp_count += 1;
        return format!("t{}", self.temp_count)
    }

    /// Semantic cube: given two types and an operation, returns the resulting operation
    pub fn obtain_type(left_type : &str, right_type : &str, operand : &str) -> Result<String, String> {
        let type_index: HashMap<&str, usize> = HashMap::from([
        (ENTERO_TYPE,   0),
        (FLOTANTE_TYPE, 1),
        ]);

        let op_index: HashMap<&str, usize> = HashMap::from([
            ("+",  0), ("-",  1), ("*",  2), ("/",  3),
            (">",  4), ("<",  5), ("!=", 6), ("==", 7),
        ]);

        // This semantic cube is a 2x2x8 which matches the types and operands
        static SEMANTIC_CUBE : [[[&str; 8]; 2]; 2] = [
            // left = entero
            [
                // right = entero
                [ENTERO_TYPE,   ENTERO_TYPE,   ENTERO_TYPE,   ENTERO_TYPE,   ENTERO_TYPE, ENTERO_TYPE, ENTERO_TYPE, ENTERO_TYPE],
                // right = flotante
                [FLOTANTE_TYPE, FLOTANTE_TYPE, FLOTANTE_TYPE, FLOTANTE_TYPE, ENTERO_TYPE, ENTERO_TYPE, ENTERO_TYPE, ENTERO_TYPE],
            ],
            // left = flotante
            [
                // right = entero
                [FLOTANTE_TYPE, FLOTANTE_TYPE, FLOTANTE_TYPE, FLOTANTE_TYPE, ENTERO_TYPE, ENTERO_TYPE, ENTERO_TYPE, ENTERO_TYPE],
                // right = flotante
                [FLOTANTE_TYPE, FLOTANTE_TYPE, FLOTANTE_TYPE, FLOTANTE_TYPE, ENTERO_TYPE, ENTERO_TYPE, ENTERO_TYPE, ENTERO_TYPE],
            ],
        ];

        let left_indx : usize = match type_index.get(left_type) {
            Some(&val) => val, // de-reference the value with &
            None => return Err(format!("Invalid type")), // Ends early the whole function
        };
        let right_indx : usize = match type_index.get(right_type) {
            Some(&val) => val,
            None => return Err(format!("Invalid type")),
        };
        let operand_indx : usize  = match op_index.get(operand) {
            Some(&val) => val,
            None => return Err(format!("Invalid type")),
        };

        Ok(SEMANTIC_CUBE[left_indx][right_indx][operand_indx].to_string())
    }

    //* Getter and setter methods */
    pub fn push_var(&mut self, name: String, var_type: String) { self.stack_vars.push((name, var_type)); }

    pub fn push_op(&mut self, op: String) { self.stack_op.push(op); }

    /// Saves a jump in the last part of the array
    pub fn push_jump(&mut self) {
        self.stack_jump.push(self.quads.len()) 
    }

    pub fn add_gotof(&mut self) {
        self.push_jump(); // Creates a checkpoint saying that this needs to be revisited
        let condition_result = self.stack_vars.pop().unwrap();
        self.quads.push(Quad { operator: GOTOF.to_string(), left: condition_result.0, right: "".to_string(), result: "".to_string() });
    }

    pub fn update_gotox(&mut self) {
        let jump_indx = self.stack_jump.pop().unwrap();
        self.quads[jump_indx].result = format!("{}", self.quads.len());
    }

    pub fn add_goto_if(&mut self) {
        let goto_index = self.quads.len();
        self.quads.push(Quad { operator: GOTO.to_string(), left: "".to_string(), right: "".to_string(), result: "".to_string() }); // Add the goto
        self.update_gotox(); // update the current GotoF pending
        self.stack_jump.push(goto_index); // Creates a checkpoint saying that this needs to be revisited (use goto_index because push_jump would use one more than current one)
    }

    /// Manage the final goto that exists at the end of the while
    pub fn add_goto_while(&mut self) {
        let gotof_indx = self.stack_jump.pop().unwrap(); // gotoF of the while
        let expression_indx = self.stack_jump.pop().unwrap(); // where the expression of while starts

        self.quads.push(Quad { operator: GOTO.to_string(), left: "".to_string(), right: "".to_string(), result: format!("{}", expression_indx) });
        self.quads[gotof_indx].result = format!("{}", self.quads.len());
    }

    pub fn pop_op(&mut self) { self.stack_op.pop(); }

    pub fn stack_op_last(&self) -> Option<&String> {
        self.stack_op.last()
    }
}
