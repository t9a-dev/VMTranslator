use anyhow::Result;
use std::result::Result::Ok;
use std::{
    fs::File,
    io::{BufRead, BufReader},
};

const COMMENT_OUT_TOKEN: &str = "//";
const ARITHMETIC_COMMANDS: [&str; 9] = ["add", "sub", "neg", "eq", "gt", "lt", "and", "or", "not"];
const PUSH_COMMAND: &str = "push";
const POP_COMMAND: &str = "pop";

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
    pub fn new(filename: &str) -> Self {
        Self {
            vm_code: Box::new(BufReader::new(File::open(filename).unwrap())),
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

        if ARITHMETIC_COMMANDS
            .iter()
            .any(|arithmetic_command| command.starts_with(*arithmetic_command))
        {
            return Ok(Some(CommandType::Arithmetic));
        }

        if command.starts_with(PUSH_COMMAND) {
            return Ok(Some(CommandType::Push));
        };
        if command.starts_with(POP_COMMAND) {
            return Ok(Some(CommandType::Pop));
        };

        Ok(None)
    }

    pub fn arg1(&self) -> Result<String> {
        let current_command = self
            .current_command
            .clone()
            .expect("current command is empty");
        let commands = current_command.split_whitespace();
        match self.command_type()?.unwrap() {
            CommandType::Arithmetic => Ok(commands.into_iter().nth(0).unwrap().to_string()),
            CommandType::Push | CommandType::Pop => {
                Ok(commands.into_iter().nth(1).unwrap().to_string())
            }
            _ => todo!(),
        }
    }

    pub fn arg2(&self) -> Result<Option<u16>> {
        let current_command = self
            .current_command
            .clone()
            .expect("current command is empty");
        let commands = current_command.split_whitespace();
        match self.command_type()?.unwrap() {
            CommandType::Push | CommandType::Pop => {
                Ok(Some(commands.into_iter().nth(2).unwrap().parse()?))
            }
            _ => Ok(None),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{fs, io::Write, path::Path};

    use super::*;
    use rand::distr::{Alphanumeric, SampleString};

    fn create_test_file(file_content: &str) -> String {
        let filename = Alphanumeric.sample_string(&mut rand::rng(), 5);
        //bacon testでファイル変更検知が発生しないようにtargetディレクトリにテストファイルを作成する。
        let _ = fs::create_dir_all("../target/test/data");
        let file_path = Path::new("../target/test/data").join(&filename);
        let mut file = File::create(&file_path).unwrap();
        file.write(file_content.as_bytes()).unwrap();

        file_path.to_string_lossy().to_string()
    }

    #[test]
    fn test_constructor() {
        let test_file = create_test_file("");
        let parser = Parser::new(&test_file);
        parser
            .vm_code
            .lines()
            .into_iter()
            .for_each(|line| println!("{}", line.unwrap()));

        let _ = fs::remove_file(test_file);
    }

    #[test]
    fn test_has_more_lines() -> Result<()> {
        let file_content = r#"
        // push
        push constant 7
        push constant 8
        add
       "#;
        let test_file = create_test_file(&file_content);

        let mut parser = Parser::new(&test_file);
        let _ = fs::remove_file(test_file);

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
        let test_file = create_test_file(&file_content);

        let mut parser = Parser::new(&test_file);
        let _ = fs::remove_file(test_file);

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
        let test_file = create_test_file(&file_content);

        let mut parser = Parser::new(&test_file);
        let _ = fs::remove_file(test_file);

        parser.advance()?;
        assert_eq!(parser.arg1()?, "constant".to_string());
        assert_eq!(parser.arg2()?.unwrap(), "7".to_string());

        parser.advance()?;
        assert_eq!(parser.arg1()?, "constant".to_string());
        assert_eq!(parser.arg2()?.unwrap(), "8".to_string());

        parser.advance()?;
        assert_eq!(parser.arg1()?, "add".to_string());
        assert_eq!(parser.arg2()?, None);

        parser.advance()?;
        assert_eq!(parser.arg1()?, "constant".to_string());
        assert_eq!(parser.arg2()?.unwrap(), "7".to_string());

        parser.advance()?;
        assert_eq!(parser.arg1()?, "constant".to_string());
        assert_eq!(parser.arg2()?.unwrap(), "8".to_string());

        parser.advance()?;
        assert_eq!(parser.arg1()?, "this".to_string());
        assert_eq!(parser.arg2()?.unwrap(), "0".to_string());

        parser.advance()?;
        assert_eq!(parser.arg1()?, "this".to_string());
        assert_eq!(parser.arg2()?.unwrap(), "5".to_string());

        Ok(())
    }
}
