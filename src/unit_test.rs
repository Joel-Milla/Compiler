#[cfg(test)] // se usa el formato cfg(test) sugerido por claude, el cual es oficial de rust
mod tests {
    use crate::CSVParser;
    use pest::Parser;
    use crate::Rule;

    #[test]
    fn test_valid_program() {
        let program = CSVParser::parse(Rule::PROGRAM, 
        "
            programa miPrograma;
            vars
                x, y : entero;
                z : flotante;
            flotante function1(id1:entero, sincero:flotante, hola:entero){
                vars
                    var1, x,y : flotante;
                {
                    asignacion = 2 + 4;
                    mientras(true) haz {escribe('hola');};
                }
            };
            inicio
                {
                    x = 5;
                    y = x + 3;
                    si (x > y) {
                        x = x + 1;
                    } sino {
                        y = y + 1;
                    };
                    mientras (x < 10) haz {
                        x = x + 1;
                    };
                    escribe ('resultado:', x);
                }
            fin
        "
        );
        assert!(program.is_ok());
    }
    #[test]
    fn test_invalid_program() {
        // El error en el siguiente test es debido a que le falta al condicional una llave
        let program = CSVParser::parse(Rule::PROGRAM, 
    "
            programa miPrograma;
            vars
                x : entero;
            inicio
                {
                    x = 5;
                    si (x > 3) {
                        x = x + 1;
                    
                }
            fin
            "
        );
        assert!(program.is_err());
    }

    #[test]
    fn test_multiple_estatutos() {
        let multiple_declartions = CSVParser::parse(Rule::TEST_ESTATUTO,"[
                            id = 32;
                            mientras (true + false) haz {escribe('true');};
                            si (false) {llamar(externa, funcion);}
                                sino{escribe(fallado);};
                        ]");
        assert!(multiple_declartions.is_ok());
    }

    #[test]
    fn test_valid_print() {
        let valid_print = CSVParser::parse(Rule::TEST_IMPRIME,"escribe ('letrero grande', 43<8, 'otro string', variable);");
        assert!(valid_print.is_ok());
    }

    #[test]
    fn test_invalid_print() {
        let lacking_separator = CSVParser::parse(Rule::TEST_IMPRIME,"escribe (letrero grande', 43<8, 'otro string');");
        let wrong_keyword = CSVParser::parse(Rule::TEST_IMPRIME,"escribir (letrero grande', 43<8, 'otro string');");
        assert!(lacking_separator.is_err());
        assert!(wrong_keyword.is_err());
    }
    
    #[test]
    fn test_valid_funcs() {
        let valid_function = CSVParser::parse(Rule::TEST_FUNCS,"
                                flotante function1(id1:entero, sincero:flotante, hola:entero){
                                    vars
                                        var1, x,y : flotante;
                                    {
                                        asignacion = 2 + 4;
                                        mientras(true) haz {escribe('hola');};
                                    }
                                };
        ");
        let no_vars_no_parameters = CSVParser::parse(Rule::TEST_FUNCS,"
        nula function1(){
            {
            asignacion = 2 + 4;
            mientras(true) haz {escribe('hola');};
            }
        };
        ");

        assert!(valid_function.is_ok());
        assert!(no_vars_no_parameters.is_ok());
    }

    #[test]
    fn test_invalid_funcs() {
        let extra_comma = CSVParser::parse(Rule::TEST_FUNCS,"
                                entera function1(id1:entero, ){
                                    vars
                                        var1, x,y : flotante;
                                    {
                                        asignacion = 2 + 4;
                                        mientras(true) haz {escribe('hola');};
                                    }
                                };
        ");

        assert!(extra_comma.is_err());
    }

    #[test]
    fn test_valid_cycle() {
        let valid_cycle1 = CSVParser::parse(Rule::TEST_CICLO,"
                                mientras ( (-32+4) != 4*4 ) haz {
                                    id = 32;
                                    si (verdadero) {llamada_vacia();}
                                    sino {escribe (32>4, 'texto random', 'hola');};
                                };
        ");
        
        let empty_cycle1 = CSVParser::parse(Rule::TEST_CICLO,"
                                mientras ( llamada(10)*2) haz {};
        ");

        assert!(valid_cycle1.is_ok());
        assert!(empty_cycle1.is_ok());
    }

    #[test]
    fn test_invalid_cycle() {
        let wrong_keyword_cycle = CSVParser::parse(Rule::TEST_CICLO,"
                                mientra ( (-32+4) != 4*4 ) haz {
                                    id = 32;
                                    si (verdadero) {llamada_vacia();}
                                    sino {escribe (32>4, texto random, hola);};
                                };
        ");
        let lack_punto_coma = CSVParser::parse(Rule::TEST_CICLO,"
                                mientras ( llamada(10)*2) haz {}
        ");

        assert!(wrong_keyword_cycle.is_err());
        assert!(lack_punto_coma.is_err());
    }

    #[test]
    fn test_valid_assing() {
        let valid_assign1 = CSVParser::parse(Rule::ASIGNA,"id_3hello = (32<4) != llamada(14,recursive(32));");
        let valid_assign2 = CSVParser::parse(Rule::ASIGNA,"glass_no = +32*3*9/7-3 != 12;");
        
        assert!(valid_assign1.is_ok());
        assert!(valid_assign2.is_ok());
    }

    #[test]
    fn test_invalid_assing() {
        let multiple_operators = CSVParser::parse(Rule::ASIGNA,"id_3hello = 32<4 < 5;");
        let lacking_punto_coma = CSVParser::parse(Rule::ASIGNA,"glass_no = +32*3*9/7-3 != 12");
        
        assert!(multiple_operators.is_err());
        assert!(lacking_punto_coma.is_err());
    }

    #[test]
    fn test_valid_if_else() {
        let valid_if = CSVParser::parse(Rule::TEST_CONDICION, 
            "si (llamada(4) > -42) { 
                        id = (32) > 2;
                    };
            ");
        let valid_if_else = CSVParser::parse(Rule::TEST_CONDICION, 
            "si (llamada(4) > -42) { 
                        id = (32) > 2;
                    } sino {
                        llamada_funcion(3*5,-variable,+otra_var);
                    };
            ");

        assert!(valid_if.is_ok());
        assert!(valid_if_else.is_ok());
    }

    #[test]
    fn test_invalid_if_else() {
        let missing_parenthesis = CSVParser::parse(Rule::TEST_CONDICION, 
            "si (llamada(4) > -42 { 
                        id = (32) > 2;
                    };
            ");
        let missing_bracket = CSVParser::parse(Rule::TEST_CONDICION, 
            "si (llamada(4) > -42) { 
                        id = (32) > 2;
                    } sino
                        llamada_funcion(3*5,-variable,+otra_var);
                    };
            ");
        assert!(missing_parenthesis.is_err());
        assert!(missing_bracket.is_err());
    }

    #[test]
    fn test_valid_expresion() {
        let factor_expresion = CSVParser::parse(Rule::TEST_EXPRESION, "(3 + 4)");
        let integer_num = CSVParser::parse(Rule::TEST_EXPRESION, "-4");
        let negative_variable = CSVParser::parse(Rule::TEST_EXPRESION, "-number");
        let empty_call = CSVParser::parse(Rule::TEST_EXPRESION, "random_id ()");
        let call = CSVParser::parse(Rule::TEST_EXPRESION, "random_id (-3, 54)");
        let operator1 = CSVParser::parse(Rule::TEST_EXPRESION, "random_id (-3, 54) > 4 * 3 + 5");
        let operation_with_operators = CSVParser::parse(Rule::TEST_EXPRESION, "3 * 5 -8+6 != 4 * 3 + 5");
        let operation_with_operators2 = CSVParser::parse(Rule::TEST_EXPRESION, "(3 * llamada(5-6)-termino(2)) -8+6 != 4 * 3 + 5");
        
        assert!(factor_expresion.is_ok());
        assert!(integer_num.is_ok());
        assert!(negative_variable.is_ok());
        assert!(empty_call.is_ok());
        assert!(call.is_ok());
        assert!(operator1.is_ok());
        assert!(operation_with_operators.is_ok());
        assert!(operation_with_operators2.is_ok());
    }
    
    #[test]
    fn test_invalid_expresion() {
        let no_parenthesis = CSVParser::parse(Rule::TEST_EXPRESION, "(3 + 4");
        let double_negative_number = CSVParser::parse(Rule::TEST_EXPRESION, "-4-");
        let double_negative_variable = CSVParser::parse(Rule::TEST_EXPRESION, "-number-");
        let negative_wrong_function = CSVParser::parse(Rule::TEST_EXPRESION, "random_id ()-");
        let wrong_call = CSVParser::parse(Rule::TEST_EXPRESION, "random_id (-3 54)");
        let lacks_parenthesis = CSVParser::parse(Rule::TEST_EXPRESION, "random_id (-3, 54) > (4 * 3 + 5");
        let lacks_one_operator = CSVParser::parse(Rule::TEST_EXPRESION, "3 * 5 -8+6 4 * 3 + 5");
        
        assert!(no_parenthesis.is_err());
        assert!(double_negative_number.is_err());
        assert!(double_negative_variable.is_err());
        assert!(negative_wrong_function.is_err());
        assert!(wrong_call.is_err());
        assert!(lacks_parenthesis.is_err());
        assert!(lacks_one_operator.is_err());
    }

    #[test]
    fn test_valid_variables() {
        let variables1 = CSVParser::parse(Rule::VARS, 
            "vars 
                        x : entero;
                    y : flotante;
                        x,y,a30, entero_23 : entero;
                        prueba
                            : flotante
                                        ;");
        let variables2 = CSVParser::parse(Rule::VARS, 
            "vars x : entero;
                        y : flotante;
                        x,y,a30, entero_23 : entero;
                        prueba: flotante;");

        assert!(variables1.is_ok()); // se usa assert a sugerencia de claude
        assert!(variables2.is_ok());
    }

    #[test]
    fn test_invalid_variables() {
        let variables1 = CSVParser::parse(Rule::VARS, 
            "x : entero;
                    y : flotante;
                        x,y,a30, entero_23 : entero;
                        prueba
                            : flotante
                                        ;");
        let variables2 = CSVParser::parse(Rule::VARS, 
            "vars x : entero
                        y : flotante
                        x,y,a30, entero_23 : entero;
                        prueba: flotante;");

        assert!(variables1.is_err()); // se usa assert a sugerencia de claude
        assert!(variables2.is_err());
    }

    #[test]
    fn test_valid_ids() {
        let id1 = CSVParser::parse(Rule::ID, "variables");
        let id2 = CSVParser::parse(Rule::ID, "eating_4_you2"); 
        let id3 = CSVParser::parse(Rule::ID, "a"); 

        assert!(id1.is_ok()); // se usa assert a sugerencia de claude
        assert!(id2.is_ok());
        assert!(id3.is_ok());
    }

    #[test]
    fn test_invalid_ids() {
        let id1 = CSVParser::parse(Rule::ID, "4variables");
        let id2 = CSVParser::parse(Rule::ID, "_a"); 
        let id3 = CSVParser::parse(Rule::ID, ""); 

        assert!(id1.is_err());
        assert!(id2.is_err());
        assert!(id3.is_err());
    }

    #[test]
    fn test_valid_tipos() {
        let type1 = CSVParser::parse(Rule::TYPE, "entero");
        let type2 = CSVParser::parse(Rule::TYPE, "flotante"); 

        assert!(type1.is_ok());
        assert!(type2.is_ok());
    }

    #[test]
    fn test_invalid_tipos() {
        let type1 = CSVParser::parse(Rule::TYPE, "string");
        let type2 = CSVParser::parse(Rule::TYPE, "boolean"); 

        assert!(type1.is_err());
        assert!(type2.is_err());
    }
}