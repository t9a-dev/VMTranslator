use anyhow::Result;
use unindent::unindent;
use crate::VariableRegister;

const ARITHMETIC_COMMANDS: [&str;3] = ["add","sub","neg"];
const COMPARISON_COMMANDS: [&str;3] = ["eq","gt","lt"];
const LOGICAL_COMMANDS: [&str;3] = ["and","or","not"];

pub struct ArithmeticCommandHelper {}

impl ArithmeticCommandHelper {
    pub fn get_command(
        command: &str,
        variable_register: &VariableRegister,
    ) -> Result<String> {
        match command {
            cmd if ARITHMETIC_COMMANDS.iter().any(|a_cmd| *a_cmd == cmd) => {
                Ok(Self::get_arithmetic_command(command, variable_register)?.unwrap())
            },
            cmd if COMPARISON_COMMANDS.iter().any(|c_cmd| *c_cmd == cmd) => {
                Ok(Self::get_comparison_command(command, variable_register)?.unwrap())
            },
            cmd if LOGICAL_COMMANDS.iter().any(|l_cmd| *l_cmd == cmd) => {
                Ok(Self::get_logical_command(command, variable_register)?.unwrap())
            },
            cmd => panic!("no support command: {}",cmd)
        }
    }

    fn get_arithmetic_command(
        command: &str,
        variable_register: &VariableRegister,
    ) -> Result<Option<String>> {
        let operator: Option<&str> = match command {
            "add" => Some("\n// add\nD=D+M\n"),
            "sub" => Some("\n// sub\nD=D-M\n"),
            "neg" => Some("\n// neg\nD=-M\n"),
            &_ => None,
        };

        if operator.is_none() {
            return Ok(None);
        }

        Ok(Some(unindent(
            format!("@{}\n{}", variable_register.as_ref(), operator.unwrap(),).as_str(),
        )))
    }

    fn get_comparison_command(
        command: &str,
        variable_register: &VariableRegister,
    ) -> Result<Option<String>> {
        let comp_operator: Option<&str> = match command {
            "eq" => Some("\n// eq\nD;JEQ\n"),
            "gt" => Some("\n// gt\nD;JGT\n"),
            "lt" => Some("\n// lt\nD;JLT\n"),
            &_ => None,
        };

        if comp_operator.is_none() {
            return Ok(None);
        }

        Ok(Some(unindent(
            format!(
                r#"
        @{}
        D=D-M
        @TRUE
        {}
        D=0
        @PUSH
        0;JMP
        (TRUE)
        D=-1
        (PUSH)
        "#,
                variable_register.as_ref(),
                comp_operator.unwrap(),
            )
            .as_str(),
        )))
    }

    fn get_logical_command(
        command: &str,
        variable_register: &VariableRegister,
    ) -> Result<Option<String>> {
        let operator: Option<&str> = match command {
            "and" => Some("\n// and\nD=D&M\n"),
            "or" => Some("\n// or\nD=D|M\n"),
            "not" => Some("\n// not\nD=!M\n"),
            &_ => None,
        };

        if operator.is_none() {
            return Ok(None);
        }

        Ok(Some(unindent(
            format!(
                r#"
        @{}
        {}
        "#,
                variable_register.as_ref(),
                operator.unwrap(),
            )
            .as_str(),
        )))
    }
}
