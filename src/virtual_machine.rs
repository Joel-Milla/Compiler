use std::fs;
use std::collections::HashMap;

use crate::directory::{DirFunc, FuncEntry};
use crate::quadruples::Quad;
use crate::constants::{ ASSIGN, CTE_FLOAT_START, CTE_INT_START, CTE_LITERAL_START, DIV, END, END_F, ENTERO_TYPE, EQUAL, ERA, GLOBAL, GOSUB, GOTO, GOTOF, LESS, LOCAL_FLOAT_START, LOCAL_INT_START, MORE, MULTP, NOT_EQUAL, PARAM, PLUS, PRINT, RETURN, SUBS };

/// One runtime value. The address tells us the scope, but each cell still has
/// to carry its own type so a single map can hold ints, floats and bools.
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Int(i32),
    Float(f64),
    Str(String), // for string constants (e.g. 'fib')
}

/// A block of memory keyed by virtual address. The same type is used for the
/// global scope, the constant table, and each function's activation record
/// (local + temp), because "todas las memorias son iguales".
type Memory = HashMap<usize, Value>; // (address, value)

pub struct VirtualMachine {
    directory : DirFunc,
    quads : Vec<Quad>,
    global : Memory,
    constants : Memory,
    // Variables needed for calling stack
    call_stack : Vec<Memory>,
    return_stack : Vec<usize>,
    pending_frames : Vec<Memory>,
    pending_functs : Vec<String>,
}

impl VirtualMachine {
    /// Creates an empty Virtual Machine, ready to be filled from an obj file.
    pub fn new() -> Self {
        VirtualMachine {
            directory : DirFunc::new(),
            quads : Vec::new(),
            global : Memory::new(),
            constants : Memory::new(),
            call_stack : Vec::new(),
            return_stack : Vec::new(),
            pending_frames : Vec::new(),
            pending_functs : Vec::new(),
        }
    }

    /// Receives the path to a `.obj` file and executes the actions
    pub fn execute(&mut self, file : &str) -> Result<(), String> {
        self.fill_values_with(file)?;
        self.set_constants()?;
        self.call_stack.push(Memory::new());
 
        let mut ip : usize = self.directory.functions.get(GLOBAL).unwrap().starting_quad; // execution pointer

        // Run until the end
        while ip < self.quads.len() {
            let operator = self.quads[ip].operator.clone();
            let left     = self.quads[ip].left.clone();
            let right    = self.quads[ip].right.clone();
            let result   = self.quads[ip].result.clone();

            // Arithmetic operators
            if operator == PLUS || operator == SUBS || operator == MULTP || operator == DIV {
                self.arithmetic(&left, &right, &result, &operator)?;
                ip += 1;

            // Relational operators
            } else if operator == LESS || operator == MORE || operator == EQUAL || operator == NOT_EQUAL {
                self.relational(&left, &right, &result, &operator)?;
                ip += 1;

            // Assign operator
            } else if operator == ASSIGN {
                // result = left
                let left_address = self.get_address(left)?;
                let left_val = self.get_value(left_address)?;
                let result_address = self.get_address(result)?;
                self.set_value(result_address, left_val)?;

                ip += 1;

            // Jumps
            } else if operator == GOTO {
                // unconditional jump: ip = result
                let result = self.get_address(result)?; // get what is in result, which is the quad line to go to
                ip = result;

            } else if operator == GOTOF {
                // if value at `left` is false: ip = result ; else ip += 1
                let left_address = self.get_address(left)?;
                let left_val = self.get_value(left_address)?;
                
                if let Value::Int(val) = left_val {
                    if val == 0 {
                        let result = self.get_address(result)?; // get what is in result, which is the quad line to go to
                        ip = result;
                    } else {
                        ip += 1;
                    }
                } else {
                    return Err(format!("Trying to use float variable as a boolean"));
                }
            } else if operator == PRINT {
                let address = self.get_address(result)?;
                let value = self.get_value(address)?;
                match value {
                    Value::Int(n) => println!("{}", n),
                    Value::Float(f) => println!("{}", f),
                    Value::Str(s) => println!("{}", s),
                }
                ip += 1;

            } else if operator == ERA {
                // ERA = ERA _ _ FUNCT_NAME

                // `result` = function name: start a new activation record.
                // Push it so nested calls (a call inside another call's argument)
                // each keep their own pending frame.
                self.pending_frames.push(Memory::new());
                self.pending_functs.push(result.clone()); // `result` is the function name

                ip += 1;
            } else if operator == PARAM {
                // PARAM = PARAM ADDRESS_VAR _ PARAM_INDX

                // copy value at `left` into parameter #`result` of the frame
                // currently being built (the top of the pending stack).
                let arg_address = self.get_address(left)?;
                let arg_value = self.get_value(arg_address)?;

                let param_number = result.parse::<usize>()
                    .map_err(|_| format!("Invalid param number '{}'", result))?;

                let funct_name = self.pending_functs.last()
                    .ok_or_else(|| format!("PARAM with no pending function"))?
                    .clone();
                let param_address = self.param_address(&funct_name, param_number)?;

                let frame = self.pending_frames.last_mut()
                    .ok_or_else(|| format!("PARAM with no pending frame"))?;
                frame.insert(param_address, arg_value);
                ip += 1;
            } else if operator == GOSUB {
                // GOSUB = GOSUB _ _ funct_name

                // `result` = function name: push return address (ip + 1), switch
                // to the new record, and jump to that function's starting_quad.
                let function = self.directory.functions.get(&result)
                    .ok_or_else(|| format!("Function '{}' doesn't exist", result))?; // get function data
                let start = function.starting_quad;

                // Save where to come back to (the quad AFTER this gosub).
                self.return_stack.push(ip + 1);

                // Pop the pending frame (top of the stack) and make it active.
                let frame = self.pending_frames.pop()
                    .ok_or_else(|| format!("GOSUB with no pending frame"))?;
                self.pending_functs.pop(); // discard the matching name
                self.call_stack.push(frame);

                ip = start;

            } else if operator == RETURN {
                // RETURN = RETURN ADDRESS_TEMP _ GLOBAL_ADDRESS

                // copy value at `left` into the function's global return slot (`result`).
                let value_address = self.get_address(left)?;
                let return_value = self.get_value(value_address)?;

                let result_address = self.get_address(result)?;
                self.set_value(result_address, return_value)?;

                // The function is done: pop its frame and jump back to the caller.
                self.call_stack.pop();
                ip = self.return_stack.pop()
                    .ok_or_else(|| format!("Return with no caller to go back to"))?;
            } else if operator == END_F {
                // end of a function body: pop the activation record and jump back
                // to the saved return address.
                self.call_stack.pop();
                ip = self.return_stack.pop()
                    .ok_or_else(|| format!("End of function with no caller to go back to"))?;

            } else if operator == END {
                break;
            } else {
                return Err(format!("Unknown operator '{}' at quad {}", operator, ip));
            }
        }

        Ok(())
    }

    /// Helper function
    fn get_address(&self, address : String) -> Result<usize, String> {
        address.parse::<usize>().map_err(|_| format!("Invalid address '{}'", address))
    }

    /// Does a binary arithmetic op with the 4 possible combinations (int or float)
    fn arithmetic(&mut self, left: &str, right: &str, result: &str, op: &str) -> Result<(), String> {
        let left_val  = self.get_value(self.get_address(left.to_string())?)?;
        let right_val = self.get_value(self.get_address(right.to_string())?)?;
        let result_address = self.get_address(result.to_string())?;

        // Match all possible combinations
        // ! AI Helped me on generating this code for all possible combinations. Same for relational stuff
        let result_val = match (left_val, right_val) {
            (Value::Int(a),   Value::Int(b))   => match op {
                PLUS => Value::Int(a + b), SUBS => Value::Int(a - b),
                MULTP => Value::Int(a * b), DIV => Value::Int(a / b),
                _ => return Err(format!("Unknown arithmetic op '{}'", op)),
            },
            (Value::Float(a), Value::Float(b)) => match op {
                PLUS => Value::Float(a + b), SUBS => Value::Float(a - b),
                MULTP => Value::Float(a * b), DIV => Value::Float(a / b),
                _ => return Err(format!("Unknown arithmetic op '{}'", op)),
            },
            // mixed → promote int to float, result is float (matches your cube)
            (Value::Int(a),   Value::Float(b)) => match op {
                PLUS => Value::Float(a as f64 + b), SUBS => Value::Float(a as f64 - b),
                MULTP => Value::Float(a as f64 * b), DIV => Value::Float(a as f64 / b),
                _ => return Err(format!("Unknown arithmetic op '{}'", op)),
            },
            (Value::Float(a), Value::Int(b))   => match op {
                PLUS => Value::Float(a + b as f64), SUBS => Value::Float(a - b as f64),
                MULTP => Value::Float(a * b as f64), DIV => Value::Float(a / b as f64),
                _ => return Err(format!("Unknown arithmetic op '{}'", op)),
            },
            _ => return Err(format!("Type mismatch in op '{}'", op)),
        };

        self.set_value(result_address, result_val)
    }

    /// Does a binary comparison: reads `left` and `right`, applies `op`,
    /// and stores the boolean result at `result`.
    fn relational(&mut self, left: &str, right: &str, result: &str, op: &str) -> Result<(), String> {
        let left_val  = self.get_value(self.get_address(left.to_string())?)?;
        let right_val = self.get_value(self.get_address(right.to_string())?)?;
        let result_address = self.get_address(result.to_string())?;

        // ! AI Helped me on generating this code for all possible combinations. Same for relational stuff
        let outcome = match (left_val, right_val) {
            (Value::Int(a), Value::Int(b)) => match op {
                LESS => a < b, MORE => a > b, EQUAL => a == b, NOT_EQUAL => a != b,
                _ => return Err(format!("Unknown relational op '{}'", op)),
            },
            (Value::Float(a), Value::Float(b)) => match op {
                LESS => a < b, MORE => a > b, EQUAL => a == b, NOT_EQUAL => a != b,
                _ => return Err(format!("Unknown relational op '{}'", op)),
            },
            (Value::Int(a), Value::Float(b)) => match op {
                LESS => (a as f64) < b, MORE => (a as f64) > b,
                EQUAL => (a as f64) == b, NOT_EQUAL => (a as f64) != b,
                _ => return Err(format!("Unknown relational op '{}'", op)),
            },
            (Value::Float(a), Value::Int(b)) => match op {
                LESS => a < (b as f64), MORE => a > (b as f64),
                EQUAL => a == (b as f64), NOT_EQUAL => a != (b as f64),
                _ => return Err(format!("Unknown relational op '{}'", op)),
            },
            _ => return Err(format!("Type mismatch in relational op '{}'", op)),
        };

        if outcome {
            self.set_value(result_address, Value::Int(1))
        } else {
            self.set_value(result_address, Value::Int(0))
        }
    }

    /// Get a specific value from memory
    fn get_value(&self, address : usize) -> Result<Value, String> {

        // Memory is either global, constant, or local (temp or local)
        let memory = if address < CTE_INT_START {
            &self.global
        } else if address < LOCAL_INT_START {
            &self.constants
        } else {
            self.call_stack.last()
                .ok_or_else(|| format!("No active call frame to read address {}", address))?
        };

        // Copy the value that is in memory and return it
        memory.get(&address)
            .cloned()
            .ok_or_else(|| format!("Address {} has no value", address))
    }

    // Save a value in a specific memory
    fn set_value(&mut self, address : usize, value : Value) -> Result<(), String> {
        if address < CTE_INT_START {
            self.global.insert(address, value);
        } else if address < LOCAL_INT_START {
            return Err(format!("Cannot write into constant address {}", address));
        } else {
            let frame = self.call_stack.last_mut()
                .ok_or_else(|| format!("No active call frame to write address {}", address))?;
            frame.insert(address, value);
        }
        Ok(())
    }

    /// Traverse the constants and save them in the memory array
    pub fn set_constants(&mut self) -> Result<(), String> { 
        for (value, address) in self.directory.constants.clone() {
            if address >= CTE_INT_START && address < CTE_FLOAT_START {
                // int
                let parsed = value.parse::<i32>().map_err(|_| format!("Bad int constant '{}'", value))?;
                self.constants.insert(address, Value::Int(parsed));
            } else if address >= CTE_FLOAT_START && address < CTE_LITERAL_START {
                // float
                let parsed = value.parse::<f64>().map_err(|_| format!("Bad float constant '{}'", value))?;
                self.constants.insert(address, Value::Float(parsed));
            } else if address >= CTE_LITERAL_START && address < LOCAL_INT_START {
                // literal
                self.constants.insert(address, Value::Str(value.clone()));
            } else {
                return Err(format!("Wrong addresses for the constant"));
            }
        }

        Ok(())
    }

    /// Given a function name and a 1-based parameter number, return the address of the param
    fn param_address(&self, funct_name: &str, param_number: usize) -> Result<usize, String> {
        let function = self.directory.functions.get(funct_name)
            .ok_or_else(|| format!("Function '{}' doesn't exist", funct_name))?;

        let mut int_count = 0;
        let mut float_count = 0;

        // Walk the params in order until we reach the one we want.
        for (i, param_type) in function.parameters.iter().enumerate() {
            let address = if param_type == ENTERO_TYPE {
                let a = LOCAL_INT_START + int_count;
                int_count += 1;
                a
            } else {
                let a = LOCAL_FLOAT_START + float_count;
                float_count += 1;
                a
            };

            // param_number is 1-based, i is 0-based.
            if i + 1 == param_number {
                return Ok(address);
            }
        }

        Err(format!("Function '{}' has no parameter #{}", funct_name, param_number))
    }

    // ! Code below is only to read the file and fill the values of the struct. This is generated with AI
    /// Reads the obj file at `file` and fills the directory (functions +
    /// constants) and the quadruples vector with what it finds.
    ///
    /// The obj file is split into sections (see ObjWriter):
    ///   %%CONSTANTS   value|address
    ///   %%FUNCTIONS   name|type|start_quad|int|float|temp_int|temp_float|params(csv)
    ///   %%QUADRUPLES  index|operator|left|right|result
    ///   %%END
    pub fn fill_values_with(&mut self, file : &str) -> Result<(), String> {
        let content = fs::read_to_string(file)
            .map_err(|e| format!("Could not read obj file '{}': {}", file, e))?;

        // Which section we are currently reading.
        let mut section = "";

        for line in content.lines() {
            // Skip blank lines.
            if line.is_empty() {
                continue;
            }

            // A section header switches the parsing mode.
            if line.starts_with("%%") {
                section = line;
                continue;
            }

            match section {
                "%%CONSTANTS"  => self.parse_constant_line(line)?,
                "%%FUNCTIONS"  => self.parse_function_line(line)?,
                "%%QUADRUPLES" => self.parse_quadruple_line(line)?,
                _ => {} // %%END or anything unexpected: ignore.
            }
        }

        Ok(())
    }

    /// Parse a line of the %%CONSTANTS section: value|address
    fn parse_constant_line(&mut self, line : &str) -> Result<(), String> {
        let parts : Vec<&str> = line.split('|').collect();
        if parts.len() != 2 {
            return Err(format!("Malformed constant line: '{}'", line));
        }

        let value = parts[0].to_string();
        let address = parts[1].parse::<usize>()
            .map_err(|_| format!("Invalid constant address in line: '{}'", line))?;

        self.directory.constants.insert(value, address);
        Ok(())
    }

    /// Parse a line of the %%FUNCTIONS section:
    /// name|type|start_quad|int|float|temp_int|temp_float|params(csv)
    fn parse_function_line(&mut self, line : &str) -> Result<(), String> {
        let parts : Vec<&str> = line.split('|').collect();
        if parts.len() != 8 {
            return Err(format!("Malformed function line: '{}'", line));
        }

        let name = parts[0].to_string();

        let mut entry = FuncEntry::new(parts[1]); // func_type
        entry.starting_quad   = Self::to_usize(parts[2], line)?;
        entry.int_count       = Self::to_usize(parts[3], line)?;
        entry.float_count     = Self::to_usize(parts[4], line)?;
        entry.temp_int_count  = Self::to_usize(parts[5], line)?;
        entry.temp_float_count = Self::to_usize(parts[6], line)?;

        // params is a comma-separated list; may be empty.
        if !parts[7].is_empty() {
            entry.parameters = parts[7].split(',').map(|s| s.to_string()).collect();
        }

        self.directory.functions.insert(name, entry);
        Ok(())
    }

    /// Parse a line of the %%QUADRUPLES section:
    /// index|operator|left|right|result
    fn parse_quadruple_line(&mut self, line : &str) -> Result<(), String> {
        // Split into at most 5 fields. We don't limit, because operands never
        // contain '|', so a plain split is safe.
        let parts : Vec<&str> = line.split('|').collect();
        if parts.len() != 5 {
            return Err(format!("Malformed quadruple line: '{}'", line));
        }

        // parts[0] is the index; we rely on file order so we don't need it,
        // but we validate it matches the position we're inserting at.
        let index = Self::to_usize(parts[0], line)?;
        if index != self.quads.len() {
            return Err(format!(
                "Quadruple out of order: expected index {}, got {} in line '{}'",
                self.quads.len(), index, line
            ));
        }

        let quad = Quad::new(
            parts[1].to_string(), // operator
            parts[2].to_string(), // left
            parts[3].to_string(), // right
            parts[4].to_string(), // result
        );

        self.quads.push(quad);
        Ok(())
    }

    /// Small helper to parse a usize and give a clear error pointing at the line.
    fn to_usize(value : &str, line : &str) -> Result<usize, String> {
        value.parse::<usize>()
            .map_err(|_| format!("Invalid number '{}' in line: '{}'", value, line))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compiler::Compiler;
    use crate::obj_writer::ObjWriter;


    // ! Tests generated with AI

    /// Compiles a program that prints the first 6 Fibonacci numbers
    /// (0, 1, 1, 2, 3, 5) using an iterative loop, writes the obj file,
    /// and runs it through the Virtual Machine.
    /// Run with `cargo test -- --nocapture` to see the printed numbers.
    #[test]
    fn test_vm_prints_first_six_fibonacci() {
        // a, b: current and next fib ; i: counter
        let mut compiler = Compiler::new();
        compiler.compile_program("
            programa fib6;
            vars
                a, b, t, i : entero;
            inicio {
                a = 0;
                b = 1;
                i = 0;
                mientras (i < 6) haz {
                    escribe(a);
                    t = a + b;
                    a = b;
                    b = t;
                    i = i + 1;
                };
            }
            fin
        ").unwrap();

        // Write the obj file.
        let path = std::env::temp_dir().join("test_fib6.obj");
        let path_str = path.to_str().unwrap();
        ObjWriter::new(&compiler).write_to_file(path_str).unwrap();

        // Run it through the Virtual Machine. Prints 0 1 1 2 3 5.
        let mut vm = VirtualMachine::new();
        let outcome = vm.execute(path_str);

        assert!(outcome.is_ok(), "VM execution failed: {:?}", outcome);

        let _ = std::fs::remove_file(path_str);
    }
}
