use std::{fs::File, io::{BufRead, BufReader}};

use anyhow::Result;

use parser::CommandType;

pub struct Code {
    assembly_file: Box<dyn BufRead>,
}

impl Code {
    pub fn new(filename: &str) -> Self {
        Self {
            assembly_file: Box::new(BufReader::new(File::open(filename).unwrap())),
        }
    }

    pub fn write_arithmetic(command: &str) -> Result<()>{
        todo!()
    }

    pub fn write_push_pop(command: CommandType,segment: &str, index: u16) -> Result<()>{
        match command {
            CommandType::Push => {

            },
            CommandType::Pop => {

            },
            _ => (),
        }
        todo!()
    }

    pub fn close(self){
        drop(self.assembly_file)
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn playground() -> Result<()> {
        let binary_string: String = vec![0, 1, 0, 1, 1, 1]
            .into_iter()
            .map(|b| (b'0' + b) as char)
            .collect();
        assert_eq!(binary_string, "010111".to_string());
        Ok(())
    }
}
