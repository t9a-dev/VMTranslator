use anyhow::Result;
use std::{
    fs::File,
    io::{BufRead, BufReader},
};

const COMMENT_OUT_TOKEN: &str = "//";

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
                Ok(line) if line.trim().starts_with(COMMENT_OUT_TOKEN) => None,   //コメント行の場合は無視
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
        todo!()
    }

    pub fn arg1() -> Result<String>{
        todo!()
    }

    pub fn arg2() -> Result<String>{
        todo!()
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
        let file_content = "@123\n//this comment\n \n(START)\nD;JGT";
        let test_file = create_test_file(&file_content);

        let mut parser = Parser::new(&test_file);
        let _ = fs::remove_file(test_file);

        //@123
        parser.advance()?;
        assert_eq!(parser.has_more_lines()?, true);

        //(START)
        parser.advance()?;
        assert_eq!(parser.has_more_lines()?, true);

        //D;JGT
        parser.advance()?;
        assert_eq!(parser.has_more_lines()?, false);

        Ok(())
    }

}
