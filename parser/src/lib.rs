use anyhow::Result;
use std::{
    io::{BufRead, BufReader}, result::Result::Ok,
};

const COMMENT_OUT_TOKEN: &str = "//";
const ARITHMETIC_COMMANDS: [&str; 9] = ["add", "sub", "neg", "eq", "gt", "lt", "and", "or", "not"];
const PUSH_COMMAND: &str = "push";
const POP_COMMAND: &str = "pop";
const LABEL_COMMAND: &str = "label";
const GOTO_COMMAND: &str = "goto";
const IF_COMMAND: &str = "if-goto";
const FUNCTION_COMMAND: &str = "function";
const RETURN_COMMAND: &str = "return";
const CALL_COMMAND: &str = "call";

#[derive(Debug, PartialEq)]
pub enum CommandType {
    Arithmetic,
    Push,
    Pop,
    Label,
    Goto,
    If,
    Function,
    Return,
    Call,
}

pub struct Parser {
    vm_code: Box<dyn BufRead>,
    current_command: Option<String>,
}

impl Parser {
    pub fn new<T:BufRead + 'static>(vm_file: T) -> Self {
        Self {
            vm_code: Box::new(BufReader::new(vm_file)),
            current_command: None,
        }
    }

    pub fn has_more_lines(&mut self) -> Result<bool> {
        Ok(self.vm_code.fill_buf()?.iter().next().is_some())
    }

    pub fn advance(&mut self) -> Result<()> {
        // //で始まるコメント行と空白を無視して次の行を読み込む
        while self.has_more_lines()? {
            self.current_command = match self.vm_code.as_mut().lines().next().unwrap() {
                Ok(line) if line.chars().all(char::is_whitespace) => None, //空白の場合は無視
                Ok(line) if line.trim().starts_with(COMMENT_OUT_TOKEN) => None, //コメント行の場合は無視
                Ok(line) => Some(line.trim().to_string()),
                Err(_) => None,
            };
            if self.current_command.is_some() {
                break;
            }
        }
        Ok(())
    }

    pub fn command_type(&self) -> Result<Option<CommandType>> {
        let command = self.current_command.clone().expect("current command empty");
        match command {
            cmd if ARITHMETIC_COMMANDS
                .iter()
                .any(|arithmetic_command| cmd.starts_with(*arithmetic_command)) =>
            {
                Ok(Some(CommandType::Arithmetic))
            }
            cmd if cmd.starts_with(PUSH_COMMAND) => Ok(Some(CommandType::Push)),
            cmd if cmd.starts_with(POP_COMMAND) => Ok(Some(CommandType::Pop)),
            cmd if cmd.starts_with(LABEL_COMMAND) => Ok(Some(CommandType::Label)),
            cmd if cmd.starts_with(GOTO_COMMAND) => Ok(Some(CommandType::Goto)),
            cmd if cmd.starts_with(IF_COMMAND) => Ok(Some(CommandType::If)),
            cmd if cmd.starts_with(FUNCTION_COMMAND) => Ok(Some(CommandType::Function)),
            cmd if cmd.starts_with(RETURN_COMMAND) => Ok(Some(CommandType::Return)),
            cmd if cmd.starts_with(CALL_COMMAND) => Ok(Some(CommandType::Call)),
            _ => Ok(None),
        }
    }

    pub fn arg1(&self) -> Result<String> {
        let current_command = self
            .current_command
            .clone()
            .expect("current command is empty");
        let commands = current_command.split_whitespace();
        match self.command_type()?.unwrap() {
            CommandType::Arithmetic => Ok(commands.into_iter().nth(0).unwrap().to_string()),
            CommandType::Push
            | CommandType::Pop
            | CommandType::Label
            | CommandType::Goto
            | CommandType::If
            | CommandType::Function
            | CommandType::Call => Ok(commands
                .clone()
                .into_iter()
                .nth(1)
                .expect(&format!("get nth 1 failed commands: {:?}", commands))
                .to_string()),
            _ => panic!("error parse command arg1: {:?}", commands),
        }
    }

    pub fn arg2(&self) -> Result<Option<u16>> {
        let current_command = self
            .current_command
            .clone()
            .expect("current command is empty");
        let commands = current_command.split_whitespace();
        match self.command_type()?.unwrap() {
            CommandType::Push | CommandType::Pop | CommandType::Function | CommandType::Call => {
                Ok(Some(commands.into_iter().nth(2).unwrap().parse()?))
            }
            _ => panic!("error parse command arg2: {:?}", commands),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::*;

    #[test]
    fn test_constructor() {
        let parser = Parser::new(Cursor::new(b""));
        parser
            .vm_code
            .lines()
            .into_iter()
            .for_each(|line| println!("{}", line.unwrap()));
    }

    #[test]
    fn test_has_more_lines() -> Result<()> {
        let file_content = r#"
        // push
        push constant 7
        push constant 8
        add
       "#;
        let mut parser = Parser::new(Cursor::new(file_content.as_bytes()));

        parser.advance()?;
        assert_eq!(parser.has_more_lines()?, true);

        parser.advance()?;
        assert_eq!(parser.has_more_lines()?, true);

        parser.advance()?;
        assert_eq!(parser.has_more_lines()?, true);

        parser.advance()?;
        assert_eq!(parser.has_more_lines()?, false);

        Ok(())
    }

    #[test]
    fn test_command_type() -> Result<()> {
        let file_content = r#"
        // push
        push constant 7
        push constant 8
        add
        push constant 7
        push constant 8
        pop this 0
        pop this 5
       "#;
        let mut parser = Parser::new(Cursor::new(file_content.as_bytes()));

        parser.advance()?;
        assert_eq!(parser.command_type()?.unwrap(), CommandType::Push);
        parser.advance()?;
        assert_eq!(parser.command_type()?.unwrap(), CommandType::Push);

        parser.advance()?;
        assert_eq!(parser.command_type()?.unwrap(), CommandType::Arithmetic);

        parser.advance()?;
        assert_eq!(parser.command_type()?.unwrap(), CommandType::Push);

        parser.advance()?;
        assert_eq!(parser.command_type()?.unwrap(), CommandType::Push);

        parser.advance()?;
        assert_eq!(parser.command_type()?.unwrap(), CommandType::Pop);

        parser.advance()?;
        assert_eq!(parser.command_type()?.unwrap(), CommandType::Pop);

        Ok(())
    }

    #[test]
    fn test_arg1() -> Result<()> {
        let file_content = r#"
        // push
        push constant 7
        push constant 8
        add
        push constant 7
        push constant 8
        pop this 0
        pop this 5
       "#;
        let mut parser = Parser::new(Cursor::new(file_content.as_bytes()));

        parser.advance()?;
        assert_eq!(parser.arg1()?, "constant".to_string());
        assert_eq!(parser.arg2()?.unwrap(), "7".parse()?);

        parser.advance()?;
        assert_eq!(parser.arg1()?, "constant".to_string());
        assert_eq!(parser.arg2()?.unwrap(), "8".parse()?);

        parser.advance()?;
        assert_eq!(parser.arg1()?, "add".to_string());

        parser.advance()?;
        assert_eq!(parser.arg1()?, "constant".to_string());
        assert_eq!(parser.arg2()?.unwrap(), "7".parse()?);

        parser.advance()?;
        assert_eq!(parser.arg1()?, "constant".to_string());
        assert_eq!(parser.arg2()?.unwrap(), "8".parse()?);

        parser.advance()?;
        assert_eq!(parser.arg1()?, "this".to_string());
        assert_eq!(parser.arg2()?.unwrap(), "0".parse()?);

        parser.advance()?;
        assert_eq!(parser.arg1()?, "this".to_string());
        assert_eq!(parser.arg2()?.unwrap(), "5".parse()?);

        Ok(())
    }
}
