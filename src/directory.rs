use std::collections::HashMap;

use crate::constants::{CTE_FLOAT_START, CTE_INT_START, CTE_LITERAL_START, ENTERO_TYPE, FLOTANTE_TYPE, GLOBAL, GLOBAL_FLOAT_START, GLOBAL_INT_START, LITERAL_TYPE, LOCAL_FLOAT_START, LOCAL_INT_START};

// ! Claude AI helped me on define what structs should I use to architect the software eficiently.
#[derive(Debug, PartialEq)] // ParetialEq makes it so can compare two VarEntry, Debug so tests can print the error
pub struct VarEntry {
    pub var_type: String,
    pub address: usize,
}
/* 
Directions of global variables:
0 -> 999 = global int variables
1,000 -> 1,999 = global float variables
2,000 -> 2,999 = cte int variables
3,000 -> 3,999 = cte float variables
4,000 -> 4,999 = cte string variables

Directions of local variables:
5,000 -> 5,999 = local int variables
6,000 -> 6,999 = local float variables
7,000 -> 7,999 = temp int variables
8,000 -> 8,999 = temp float variables
*/

impl VarEntry {
    pub fn new(var_type : &str, address : usize) -> Self {
        VarEntry {var_type : var_type.to_string(), address : address}
    }
}

#[derive(Debug, PartialEq)]
pub struct FuncEntry {
    pub func_type: String,
    pub var_directory: HashMap<String, VarEntry>, // (address, value) of variable
    pub parameters : Vec<String>,
    pub starting_quad : usize,
    pub int_count : usize,
    pub float_count : usize,
    pub temp_int_count : usize,
    pub temp_float_count : usize,
}

impl FuncEntry {
    pub fn new(func_type : &str) -> Self {
        FuncEntry { func_type: func_type.to_string(), 
                    var_directory: HashMap::new(), 
                    parameters : Vec::new(), 
                    starting_quad : 0,
                    int_count : 0,
                    float_count : 0,
                    temp_int_count : 0,
                    temp_float_count : 0,
                }
    }

    pub fn new_with_vars (func_type : &str, vars : HashMap<String, VarEntry>, parameters : Vec<String>) -> Self {
        FuncEntry { func_type : func_type.to_string(), 
                    var_directory: vars, 
                    parameters : parameters, 
                    starting_quad : 0,
                    int_count : 0,
                    float_count : 0,
                    temp_int_count : 0,
                    temp_float_count : 0,
        }
    }

    /// Function that handles giving the address given the type
    pub fn get_address(&mut self, var_type : &str, is_global : bool) -> Result<usize, String> {
        let address;
        if is_global {
            // If is global, the addresses start at different range
            if var_type == FLOTANTE_TYPE {
                address = self.float_count + GLOBAL_FLOAT_START;
                self.float_count += 1;
            } else if var_type == ENTERO_TYPE {
                address = self.int_count + GLOBAL_INT_START;
                self.int_count += 1;
            } else {
                return Err(format!("Invalid type"));
            }
        } else {
            // If is not global, give respective start addresses
            if var_type == FLOTANTE_TYPE {
                address = self.float_count + LOCAL_FLOAT_START;
                self.float_count += 1;
            } else if var_type == ENTERO_TYPE {
                address = self.int_count + LOCAL_INT_START;
                self.int_count += 1;
            } else {
                return Err(format!("Invalid type"));
            }
        }
        Ok(address)
    }

    /// Adds a variable to the table scope of the function
    pub fn add_variable(&mut self, var_name : String, mut var_entry : VarEntry, is_global : bool) -> Result<(), String> {
        //* Throw an error if the variable already exists */
        if self.var_directory.contains_key(&var_name) {
            return Err(format!("Variable '{}' already declared", var_name))
        }
        
        let var_type = &var_entry.var_type;
        var_entry.address = self.get_address(&var_type, is_global)?;
        
        self.var_directory.insert(var_name, var_entry);
        Ok(())
    }

    /// Adds a param type to the function
    pub fn add_param(&mut self, param_type : &str) {
        self.parameters.push(param_type.to_string());
    }
}

pub struct DirFunc {
    pub functions : HashMap<String, FuncEntry>, // Name of function, and it attributes
    pub constants : HashMap<String, usize>, // (value, address) of the constant being saved
    cte_int : usize,
    cte_float : usize,
    cte_literal : usize,
}

impl DirFunc {
    pub fn new() -> Self {
        DirFunc {
            functions : HashMap::new(),
            constants : HashMap::new(),
            cte_int : 0,
            cte_float : 0,
            cte_literal : 0,
        }
    }

    /// Function to create a function in the directory
    pub fn create_function(&mut self, function_name : &str, function_type : &str) -> Result<(), String> {

        if self.functions.contains_key(function_name) {
            //* Return error because the function was already created */
            return Err(format!("Variable '{}' already declared", function_type))
            
        }
        //* Creates for the first time the function in the directory */
        self.functions.insert(function_name.to_string(), FuncEntry::new(function_type));
        
        Ok(())
    }

    /// Adds a parameters to the function signature
    pub fn add_param_to_function(&mut self, function_name : &str, param_type : &str) -> Result<(), String> {

        if !self.functions.contains_key(function_name) {
            //* Function doesn't exist, so the function was not created */
            return Err(format!("Function '{}' doesn't exist", function_name))
        }

        //* Adds the param to the function */
        self.functions.get_mut(function_name)
            .unwrap()
            .add_param(param_type);

        Ok(())

    }

    /// Add in which quad does the function start
    pub fn set_function_starting_point(&mut self, function_name : &str, starting_point : usize) -> Result<(), String> {
        if !self.functions.contains_key(function_name) {
            //* Function doesn't exist, so the function was not created */
            return Err(format!("Function '{}' doesn't exist", function_name))
        }

        self.functions.get_mut(function_name)
            .unwrap()
            .starting_quad = starting_point;

        Ok(())
    }

    /// Function in charge of adding a variable to a specific function
    pub fn add_variable_to_function(&mut self, function_name : &str, var_name : &str, var_type : &str) -> Result<(), String> {

        if !self.functions.contains_key(function_name) {
            //* Function doesn't exist, so the function was not created */
            return Err(format!("Function '{}' doesn't exist", function_name))
        }

        //* Adds the variable to the function */
        let var_entry = VarEntry { var_type : var_type.to_string(), address : 0};
        let is_global = function_name == GLOBAL;

        self.functions.get_mut(function_name)
            .unwrap()
            .add_variable(var_name.to_string(), var_entry, is_global)?;

        Ok(())
    }

    /// Get the type of a variable that exists in the function
    pub fn get_var_address_of(&self, function_name : &str, var_name : &str) -> Result<String, String> {

        //* Validate if function and variable exists */
        let mut function = self.functions.get(function_name)
            .ok_or_else(|| format!("Function '{}' doesn't exist", function_name))?; // ! AI Claude recommended way to get if exsits, or return error if not

        if let Some(var_entry) = function.var_directory.get(var_name) {
            let address = var_entry.address;
            return Ok(address.to_string());
        }

        // Fall back to GLOBAL scope to search for variable
        function = self.functions.get(GLOBAL)
            .ok_or_else(|| format!("Function '{}' doesn't exist", function_name))?;
        let var_entry = function.var_directory.get(var_name)
            .ok_or_else(|| format!("Variable '{}' doesn't exist", var_name))?;

        let address = var_entry.address;
        Ok(address.to_string())
    }

    /// Get the type of a variable that exists in the function
    pub fn get_var_type_of(&self, function_name : &str, var_name : &str) -> Result<String, String> {

        //* Validate if function and variable exists */
        let mut function = self.functions.get(function_name)
            .ok_or_else(|| format!("Function '{}' doesn't exist", function_name))?;

        if let Some(var_entry) = function.var_directory.get(var_name) {
            return Ok(var_entry.var_type.clone());
        }

        // Fall back to GLOBAL scope to search for variable
        function = self.functions.get(GLOBAL)
            .ok_or_else(|| format!("Function '{}' doesn't exist", function_name))?;
        let var_entry = function.var_directory.get(var_name)
            .ok_or_else(|| format!("Variable '{}' doesn't exist", var_name))?;

        Ok(var_entry.var_type.clone())
    }

    /// Save a constant in the constant directory
    pub fn save_constant(&mut self, var_type : &str, value : &str) -> Result<usize, String> {
        // Validate that the constant hasn't been created
        if let Some(address) = self.constants.get(value) {
            return  Ok(*address);
        };

        let address : usize;
        // Get the current count of the var_type asked
        if var_type == ENTERO_TYPE {
            address = self.cte_int + CTE_INT_START;
            self.cte_int += 1;
        } else if var_type == FLOTANTE_TYPE {
            address = self.cte_float + CTE_FLOAT_START;
            self.cte_float += 1;
        } else if var_type == LITERAL_TYPE {
            address = self.cte_literal + CTE_LITERAL_START;
            self.cte_literal += 1;
        } else {
            return Err(format!("Wrong constant type"));
        }

        self.constants.insert(value.to_string(), address);
        Ok(address)
    }

    /// Set the count of temporal int and float variables
    pub fn set_temporal_count(&mut self, function_name : &str, temp_int_count : usize, temp_float_count : usize) -> Result<(), String> {

        if !self.functions.contains_key(function_name) {
            //* Function doesn't exist, so the function was not created */
            return Err(format!("Function '{}' doesn't exist", function_name))
        }

        //* Update the temporal counts of the function */
        self.functions.get_mut(function_name)
            .unwrap()
            .temp_int_count = temp_int_count;
        self.functions.get_mut(function_name)
            .unwrap()
            .temp_float_count = temp_float_count;

        Ok(())
    }

    /// Get if a function exists in the directory
    pub fn function_exists(&self, function_name : &str) -> bool {
        return self.functions.contains_key(function_name);
    }
}