// ! AI Recommended to have external file with all the constants, to import them
//* Function and Variable types */
pub const GLOBAL : &str = "global";
pub const NULA_TYPE : &str = "nula";
pub const ENTERO_TYPE : &str = "entero";
pub const FLOTANTE_TYPE : &str = "flotante";
pub const LITERAL_TYPE : &str = "string";

//* Arithmetic operators */
pub const PLUS : &str = "+";
pub const SUBS : &str = "-";
pub const MULTP : &str = "*";
pub const DIV : &str = "/";

//* Logic operators */
pub const LESS : &str = "<";
pub const MORE : &str = ">";
pub const EQUAL : &str = "==";
pub const NOT_EQUAL : &str = "!=";
pub const ASSIGN : &str = "=";
pub const PARENTHESIS : &str = "(";

//* Quadruple rules */
pub const GOTOF : &str = "GOTO_F";
pub const GOTO : &str = "GOTO";
pub const PRINT : &str = "PRINT";

//* Where variable counting starts (start address of each memory segment) */
// Global memory
pub const GLOBAL_INT_START : usize = 0;       // 0     -> 999
pub const GLOBAL_FLOAT_START : usize = 1000;  // 1,000 -> 1,999

// Constant memory
pub const CTE_INT_START : usize = 2000;       // 2,000 -> 2,999
pub const CTE_FLOAT_START : usize = 3000;     // 3,000 -> 3,999
pub const CTE_LITERAL_START : usize = 4000;    // 4,000 -> 4,999

// Local memory
pub const LOCAL_INT_START : usize = 5000;     // 5,000 -> 5,999
pub const LOCAL_FLOAT_START : usize = 6000;   // 6,000 -> 6,999

// Temporal memory
pub const TEMP_INT_START : usize = 7000;      // 7,000 -> 7,999
pub const TEMP_FLOAT_START : usize = 8000;    // 8,000 -> 8,999

/* 
Directions of global variables:
0 -> 999 = global int variables
1,000 -> 1,999 = global float variables
2,000 -> 2,999 = cte int variables
3,000 -> 3,999 = cte float variables
4,000 -> 4,999 = cte literal variables

Directions of local variables:
5,000 -> 5,999 = local int variables
6,000 -> 6,999 = local float variables
7,000 -> 7,999 = temp int variables
8,000 -> 8,999 = temp float variables
*/
