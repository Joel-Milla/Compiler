use std::collections::HashMap;

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
    pub vars: HashMap<String, VarEntry>, // name variable, variable entry
}

impl FuncEntry {
    pub fn new(func_type : &str) -> Self {
        FuncEntry { func_type: func_type.to_string(), vars: HashMap::new() }
    }

    pub fn new_with_vars (func_type : &str, vars : HashMap<String, VarEntry>) -> Self {
        FuncEntry { func_type : func_type.to_string(), vars: vars}
    }

    /// Adds a variable to the table scope of the function
    pub fn add_variable(&mut self, var_name : String, var_entry : VarEntry) -> Result<(), String> {
        //* Throw an error if the variable already exists */
        if self.vars.contains_key(&var_name) {
            return Err(format!("Variable '{}' already declared", var_name))
        }
    
        self.vars.insert(var_name, var_entry);
        Ok(())
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

    pub fn get_var_type_of(&self, function_name : &str, var_name : &str) -> Result<String, String> {

        //* Validate if function and variable exists */
        let function = self.functions.get(function_name)
            .ok_or_else(|| format!("Function '{}' doesn't exist", function_name))?; // ! AI Claude recommended way to get if exsits, or return error if not

        let var_entry = function.vars.get(var_name)
            .ok_or_else(|| format!("Variable '{}' doesn't exist", function_name))?;

        Ok(var_entry.var_type.clone())
    }
}