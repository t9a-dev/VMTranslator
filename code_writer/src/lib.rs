use std::{fs::File, io::Write, path::Path};

use anyhow::Result;
use unindent::unindent;

use parser::CommandType;

pub struct CodeWriter {
    assembly_file: Box<dyn Write>,
    filename: String,
}

impl CodeWriter {
    pub fn new(filename: &Path) -> Self {
        Self {
            assembly_file: Box::new(File::create(filename).unwrap()),
            filename: filename.file_stem().unwrap().to_string_lossy().to_string(),
        }
    }

    pub fn write_arithmetic(command: &str) -> Result<()> {
        todo!()
    }

    pub fn write_push_pop(
        &mut self,
        command: CommandType,
        segment: &str,
        index: u16,
    ) -> Result<()> {
        match command {
            CommandType::Push => {
                self.write_segment(command, segment, index)?;
                self.write_push()?;
            }
            CommandType::Pop => {
                self.write_pop()?;
                self.write_segment(command, segment, index)?;
            }
            _ => (),
        }
        Ok(())
    }

    pub fn close(self) {
        drop(self.assembly_file)
    }

    fn write_segment(&mut self, command: CommandType, segment: &str, index: u16) -> Result<()> {
        let segment_symbol_asm = match segment {
            "local" => Some(format!("// that {}\n@LCL\n", index)),
            "argument" => Some(format!("// argument {}\n@ARG\n", index)),
            "this" => Some(format!("// this {}\n@THIS\n", index)),
            "that" => Some(format!("// that {}\n@THAT\n", index)),
            "temp" => Some(format!("// temp {}\n@TEMP\n", index)),
            "constant" => Some(format!("// constant {}\n@{}\n", index, index)),
            "pointer" if index == 0 => Some(format!("// this {}\n@THIS\n", index)),
            "pointer" if index == 1 => Some(format!("// that {}\n@THAT\n", index)),
            "static" => Some(format!(
                "// static {}\n@{}.{}\n",
                index, self.filename, index
            )),
            _ => None,
        };

        let segment_access_asm = format!(
            "{}A=M+{}\n",
            segment_symbol_asm.expect("not support segment"),
            index
        );

        match command {
            CommandType::Push => {
                self.assembly_file
                    .write(format!("{}D=M\n", segment_access_asm).as_bytes())?;
            }
            CommandType::Pop => {
                self.assembly_file
                    .write(format!("{}M=D\n", segment_access_asm).as_bytes())?;
            }
            _ => (),
        };
        Ok(())
    }

    fn write_push(&mut self) -> Result<()> {
        let push_asm = r#"// push
       @SP
       A=M
       M=D
       @SP
       M=M+1
       "#;
        self.assembly_file.write(unindent(&push_asm).as_bytes())?;
        Ok(())
    }

    fn write_pop(&mut self) -> Result<()> {
        let pop_asm = r#"// pop
       @SP
       M=M-1
       A=M
       D=M
       "#;
        self.assembly_file.write(unindent(&pop_asm).as_bytes())?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::io::Read;

    use super::*;

    #[test]
    fn playground() -> Result<()> {
        Ok(())
    }

    #[test]
    fn test_push_command() -> Result<()>{
        let test_file_name = "Test.vm";
        {
            let mut code_writer = CodeWriter::new(&Path::new(test_file_name));
            code_writer.write_push_pop(CommandType::Push,"that",5)?;
        }

        let mut asm_file_content = String::new();
        File::open(test_file_name)?.read_to_string(&mut asm_file_content)?;

        let expect_asm = r#"// that 5
        @THAT
        A=M+5
        D=M
        // push
        @SP
        A=M
        M=D
        @SP
        M=M+1
        "#;
        assert_eq!(unindent(expect_asm),unindent(&asm_file_content));

        Ok(())
    } 
}
