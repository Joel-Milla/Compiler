use crate::constants::{ASSIGN, END, END_F, ENTERO_TYPE, ERA, FLOTANTE_TYPE, GOSUB, GOTO, GOTOF, PARAM, PRINT, RETURN, TEMP_FLOAT_START, TEMP_INT_START};
use std::{collections::HashMap};

#[derive(Debug, PartialEq)]
pub struct Quad {
    operator : String,
    left : String,
    right : String,
    result : String,
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
    pub quads : Vec<Quad>, // Were all the quadruples are stored
    stack_vars : Vec<(String, String)>, // (address, type) of the variable
    stack_op : Vec<String>, // Stack to save the operators
    stack_jump : Vec<usize>,
    temp_int_count : usize,
    temp_float_count : usize,
    curr_scope : String,
}

impl Quadruples {
    pub fn new() -> Self {
        Quadruples {
            quads : Vec::new(),
            stack_vars : Vec::new(),
            stack_op : Vec::new(),
            stack_jump : Vec::new(),
            temp_int_count : 0,
            temp_float_count : 0,
            curr_scope : String::new(),
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

        let temp_type = Quadruples::obtain_type(&left.1, &right.1, &op)?;
        let temp = self.get_next_temp(&temp_type)?;

        // Add the quad to the quadruple list, and save the temporal variable to the stack
        self.quads.push(Quad::new(op, left.0, right.0,temp.clone()));
        self.push_var(temp, temp_type);

        Ok(())
    }

    /// Generate print statements into the quadruple by getting last value
    pub fn generate_print_expr(&mut self) {
        let value = self.stack_vars.pop().unwrap(); // Get the value of the last variable pushed
        self.quads.push(Quad { operator: PRINT.to_string(), left: "".to_string(), right: "".to_string(), result: value.0}); 
    }
    
    pub fn generate_print_litr(&mut self, address : &str) {
        self.quads.push(Quad { operator: PRINT.to_string(), left: "".to_string(), right: "".to_string(), result: address.to_string()});
    }

    /// Get next temporal given a type
    pub fn get_next_temp(&mut self, var_type : &str) -> Result<String, String> {
        let new_temp;
        if var_type == ENTERO_TYPE {
            new_temp = self.temp_int_count + TEMP_INT_START;
            self.temp_int_count += 1;
        } else if var_type == FLOTANTE_TYPE {
            new_temp = self.temp_float_count + TEMP_FLOAT_START;
            self.temp_float_count += 1;
        } else {
            return Err(format!("Wront type of variable"));
        }

        return Ok(new_temp.to_string());
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

    /// Function to create the ERA for function calls
    pub fn create_era(&mut self, funct_name : &str) {
        self.quads.push(Quad { operator: ERA.to_string(), left: "".to_string(), right: "".to_string(), result: funct_name.to_string()});
    }

    //* Handle the call to functions */

    /// Create the quadruple that saves the parameters of a function
    pub fn create_param(&mut self, param_counter : usize, parameters : &Vec<String>) -> Result<(), String> {
        // Param counter tells which index in the list of the parameters, is this one
        let parameter = self.stack_vars.pop().unwrap();

        // Need to validate that the parameter type being passed is the same type of the expeteced parameter. Need to also validate that didn't passed more arugments
        if param_counter > parameters.len() {
            return Err(format!("Too many arguments: parameter #{} doesn't exist", param_counter));
        }
        let expected = &parameters[param_counter - 1];
        if parameter.1 != expected.as_str() {
            return Err(format!(
                "Type mismatch on parameter #{}: expected {}, got {}",
                param_counter, expected, parameter.1
            ));
        }

        self.quads.push(Quad { operator: PARAM.to_string(), left: parameter.0, right: "".to_string(), result: param_counter.to_string() });
        Ok(())
    }

    pub fn create_gosub(&mut self, funct_name : &str) {
        self.quads.push(Quad { operator: GOSUB.to_string(), left: "".to_string(), right: "".to_string(), result: funct_name.to_string() });
    }

    /// Generate the return statement for the functions
    pub fn create_return(&mut self, funct_type : &str, global_address : &str) -> Result<(), String> {
        let var_return = self.stack_vars.pop().unwrap();

        // Check if have valid type
        if var_return.1 != funct_type {
            return Err(format!( "Return type mismatch: function expects {}, but return expression is {}", funct_type, var_return.1));
        }
        self.quads.push(Quad { operator: RETURN.to_string(), left: var_return.0, right: "".to_string(), result:  global_address.to_string()});

        Ok(())
    }

    //* Getter and setter methods */
    pub fn push_var(&mut self, address: String, var_type: String) { self.stack_vars.push((address, var_type)); }

    // This method specifically creates a quad to assign a global funct variable to a temporal variable. i.e. = fact     1000 (where fact is a global variable and 1000 its the temporal address)
    pub fn push_function_temp(&mut self, address : &str, funct_type : &str) -> Result<(), String> {
        let temp = self.get_next_temp(funct_type)?;
        self.quads.push(Quad { operator: ASSIGN.to_string(), left: address.to_string(), right: "".to_string(), result: temp.clone() });

        self.push_var(temp, funct_type.to_string()); // Add it to the stack
        Ok(())
    }

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

    pub fn get_scope(&self) -> &str {
        return &self.curr_scope;
    }
    
    pub fn set_scope_to(&mut self, scope : &str) {
        self.curr_scope = scope.to_string();
        self.temp_int_count = 0;
        self.temp_float_count = 0;
    }

    // return the counts of temporal int and float variables
    pub fn get_temporal_counts(&self) -> (usize, usize) {
        return (self.temp_int_count, self.temp_float_count)
    }

    pub fn push_end_f(&mut self) {
        self.quads.push(Quad { operator: END_F.to_string(), left: "".to_string(), right: "".to_string(), result: "".to_string() });
    }

    pub fn push_end(&mut self) {
        self.quads.push(Quad { operator: END.to_string(), left: "".to_string(), right: "".to_string(), result: "".to_string() });
    }
}
