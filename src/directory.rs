use std::collections::HashMap;

use crate::constants::GLOBAL;

// ! Claude AI helped me on define what structs should I use to architect the software eficiently.
#[derive(Debug, PartialEq)] // ParetialEq makes it so can compare two VarEntry, Debug so tests can print the error
pub struct VarEntry {
    pub var_type: String,
    pub value: String,
}

impl VarEntry {
    pub fn new(var_type : &str, value : &str) -> Self {
        VarEntry {var_type : var_type.to_string(), value : value.to_string()}
    }
}

#[derive(Debug, PartialEq)]
pub struct FuncEntry {
    pub func_type: String,
    pub var_directory: HashMap<String, VarEntry>, // name variable, variable entry
    pub parameters : Vec<String>,
    pub starting_quad : usize,
}

impl FuncEntry {
    pub fn new(func_type : &str) -> Self {
        FuncEntry { func_type: func_type.to_string(), var_directory: HashMap::new(), parameters : Vec::new(), starting_quad : 0}
    }

    pub fn new_with_vars (func_type : &str, vars : HashMap<String, VarEntry>, parameters : Vec<String>) -> Self {
        FuncEntry { func_type : func_type.to_string(), var_directory: vars, parameters : parameters, starting_quad : 0}
    }

    /// Adds a variable to the table scope of the function
    pub fn add_variable(&mut self, var_name : String, var_entry : VarEntry) -> Result<(), String> {
        //* Throw an error if the variable already exists */
        if self.var_directory.contains_key(&var_name) {
            return Err(format!("Variable '{}' already declared", var_name))
        }
    
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
}

impl DirFunc {
    pub fn new() -> Self {
        DirFunc {
            functions : HashMap::new(),
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
        let var_entry = VarEntry { var_type : var_type.to_string(), value : "".to_string()};

        self.functions.get_mut(function_name)
            .unwrap()
            .add_variable(var_name.to_string(), var_entry)?;

        Ok(())

    }

    /// Get the type of a variable that exists in the function
    pub fn get_var_type_of(&self, function_name : &str, var_name : &str) -> Result<String, String> {

        //* Validate if function and variable exists */
        let mut function = self.functions.get(function_name)
            .ok_or_else(|| format!("Function '{}' doesn't exist", function_name))?; // ! AI Claude recommended way to get if exsits, or return error if not

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
}