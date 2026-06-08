use std::fs::File;
use std::io::Write;

use crate::compiler::Compiler;
use crate::directory::FuncEntry;

// ! Asked AI to make a very simple program that gets compiler and reads it in an obj file

/// In charge of taking a finished Compiler and writing everything the Virtual
/// Machine needs (function directory, constant table and quadruples) into a
/// plain-text `.obj` file.
///
/// The file is split into sections so the VM can read it back easily:
///   %%CONSTANTS   value|address
///   %%FUNCTIONS   name|type|start_quad|int|float|temp_int|temp_float|params(csv)
///   %%QUADRUPLES  index|operator|left|right|result
///   %%END
pub struct ObjWriter<'a> {
    compiler : &'a Compiler,
}

impl<'a> ObjWriter<'a> {
    /// Receives the compiler that already finished compiling a program.
    pub fn new(compiler : &'a Compiler) -> Self {
        ObjWriter { compiler }
    }

    /// Builds the full content of the obj file and writes it to `path`.
    /// `path` should already include the `.obj` ending (e.g. "program.obj").
    pub fn write_to_file(&self, path : &str) -> Result<(), String> {
        let content = self.build_content();

        let mut file = File::create(path)
            .map_err(|e| format!("Could not create obj file '{}': {}", path, e))?;
        file.write_all(content.as_bytes())
            .map_err(|e| format!("Could not write obj file '{}': {}", path, e))?;

        Ok(())
    }

    /// Assembles every section into a single String.
    fn build_content(&self) -> String {
        let mut out = String::new();

        out.push_str(&self.constants_section());
        out.push_str(&self.functions_section());
        out.push_str(&self.quadruples_section());
        out.push_str("%%END\n");

        out
    }

    /// %%CONSTANTS — one line per constant: value|address
    fn constants_section(&self) -> String {
        let mut section = String::from("%%CONSTANTS\n");

        let constants = &self.compiler.directory.constants;

        // Sort by address so the file is deterministic (HashMap order is random).
        let mut entries : Vec<(&String, &usize)> = constants.iter().collect();
        entries.sort_by_key(|(_value, address)| **address);

        for (value, address) in entries {
            section.push_str(&format!("{}|{}\n", value, address));
        }

        section
    }

    /// %%FUNCTIONS — one line per function with its directory info and resources.
    fn functions_section(&self) -> String {
        let mut section = String::from("%%FUNCTIONS\n");

        let functions = &self.compiler.directory.functions;

        // Sort by starting_quad so global/main and the functions come out in a
        // stable, readable order.
        let mut entries : Vec<(&String, &FuncEntry)> = functions.iter().collect();
        entries.sort_by_key(|(name, func)| (func.starting_quad, (*name).clone()));

        for (name, func) in entries {
            let params = func.parameters.join(",");
            section.push_str(&format!(
                "{}|{}|{}|{}|{}|{}|{}|{}\n",
                name,
                func.func_type,
                func.starting_quad,
                func.int_count,
                func.float_count,
                func.temp_int_count,
                func.temp_float_count,
                params,
            ));
        }

        section
    }

    /// %%QUADRUPLES — one line per quad: index|operator|left|right|result
    fn quadruples_section(&self) -> String {
        let mut section = String::from("%%QUADRUPLES\n");

        for (index, quad) in self.compiler.quads.quads.iter().enumerate() {
            section.push_str(&format!(
                "{}|{}|{}|{}|{}\n",
                index,
                quad.operator,
                quad.left,
                quad.right,
                quad.result,
            ));
        }

        section
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compiler::Compiler;

    #[test]
    fn test_obj_writer_creates_file_with_sections() {
        let mut compiler = Compiler::new();
        compiler.compile_program("
            programa simple;
            vars
                a : entero;
            inicio {
                a = 5;
                escribe(a);
            }
            fin
        ").unwrap();

        let path = std::env::temp_dir().join("test_simple.obj");
        let path_str = path.to_str().unwrap();

        let writer = ObjWriter::new(&compiler);
        writer.write_to_file(path_str).unwrap();

        let written = std::fs::read_to_string(path_str).unwrap();

        // The four sections must be present.
        assert!(written.contains("%%CONSTANTS"));
        assert!(written.contains("%%FUNCTIONS"));
        assert!(written.contains("%%QUADRUPLES"));
        assert!(written.contains("%%END"));

        // The constant 5 lives at address 2000.
        assert!(written.contains("5|2000"));

        // Cleanup.
        let _ = std::fs::remove_file(path_str);
    }
}
