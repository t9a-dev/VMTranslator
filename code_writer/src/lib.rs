pub mod helper;

use std::{fs::File, io::Write, path::Path};

use anyhow::Result;
use helper::arithmetic::ArithmeticCommandHelper;
use std::convert::AsRef;
use strum_macros::AsRefStr;
use unindent::unindent;

use parser::CommandType;

#[derive(AsRefStr, Clone, Copy)]
pub enum VariableRegister {
    R13,
    R14,
    R15,
}

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

    pub fn write_arithmetic(&mut self, command: &str,comparison_count:u16) -> Result<()> {
        let is_single_operand = command == "neg" || command == "not";
        let variable_register = VariableRegister::R13;
        self.write_pop()?;
        self.write_load_register(variable_register)?;
        // オペランドが1つの場合はyのみスタックからpopする
        if !is_single_operand {
            self.write_pop()?;
        }

        // Arithmetic
        let arithmetic_command = ArithmeticCommandHelper::get_command(command, &variable_register,comparison_count)?;
        self.assembly_file.write(arithmetic_command.as_bytes())?;

        self.write_push()?;

        Ok(())
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

    pub fn close(mut self) -> Result<()> {
        self.write_infinity_loop()?;
        drop(self.assembly_file);
        Ok(())
    }

    fn write_segment(&mut self, command: CommandType, segment: &str, index: u16) -> Result<()> {
        let segment_symbol_asm = match segment {
            "local" => Some(format!("// local {}\n@LCL\n", index)),
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

        let segment_access_asm = match segment {
            "constant" => {
                format!(
                    "\n{}D=A\n",
                    segment_symbol_asm.expect("not support segment"),
                )
            },
            "static" => {
                format!(
                    "\n{}",
                    segment_symbol_asm.expect("not support segment"),
                )
            },
            _ => {
                format!(
                    "\n{}A=M+{}\n",
                    segment_symbol_asm.expect("not support segment"),
                    index
                )
            }
        };

        match command {
            CommandType::Push => {
                if segment == "constant" {
                    self.assembly_file
                        .write(format!("{}\n", segment_access_asm).as_bytes())?;
                } else {
                    self.assembly_file
                        .write(format!("{}D=M\n", segment_access_asm).as_bytes())?;
                }
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
        let push_asm = r#"

       // push
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
        let pop_asm = r#"

        // pop
        @SP
        M=M-1
        A=M
        D=M
       "#;
        self.assembly_file.write(unindent(&pop_asm).as_bytes())?;
        Ok(())
    }

    fn write_load_register(&mut self, register: VariableRegister) -> Result<()> {
        self.assembly_file
            .write(format!("@{}\nM=D\n", register.as_ref()).as_bytes())?;

        Ok(())
    }

    fn write_infinity_loop(&mut self) -> Result<()> {
        self.assembly_file.write(
            unindent(
                r#"
        (END)
        @END
        0;JMP
        "#,
            )
            .as_bytes(),
        )?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use rand::distr::{Alphanumeric, SampleString};
    use std::{fs, io::Read};

    use super::*;

    fn get_code_writer() -> Result<(CodeWriter, String)> {
        fs::create_dir_all("../target/test/data")?;
        let mut test_file_name = Alphanumeric.sample_string(&mut rand::rng(), 5);
        test_file_name = format!("{}.vm", test_file_name);
        let file_path = Path::new("../target/test/data").join(&test_file_name);
        Ok((
            CodeWriter::new(&file_path),
            file_path.to_string_lossy().to_string(),
        ))
    }

    #[test]
    fn playground() -> Result<()> {
        assert_eq!("R13", VariableRegister::R13.as_ref());
        Ok(())
    }

    #[test]
    fn test_write_segment_when_constant() -> Result<()> {
        let (mut code_writer, test_file_name) = get_code_writer()?;
        code_writer.write_segment(CommandType::Push, "constant", 10)?;

        let mut asm_file_content = String::new();
        File::open(&test_file_name)?.read_to_string(&mut asm_file_content)?;

        let expect_asm = r#"// constant 10
        @10
        D=A

        "#;
        assert_eq!(unindent(expect_asm), unindent(&asm_file_content));

        fs::remove_file(test_file_name)?;
        Ok(())
    }

    #[test]
    fn test_write_segment() -> Result<()> {
        let (mut code_writer, test_file_name) = get_code_writer()?;
        code_writer.write_segment(CommandType::Push, "that", 5)?;

        let mut asm_file_content = String::new();
        File::open(&test_file_name)?.read_to_string(&mut asm_file_content)?;

        let expect_asm = r#"// that 5
        @THAT
        A=M+5
        D=M
        "#;
        assert_eq!(unindent(expect_asm), unindent(&asm_file_content));

        fs::remove_file(test_file_name)?;
        Ok(())
    }

    #[test]
    fn test_push_command() -> Result<()> {
        let (mut code_writer, test_file_name) = get_code_writer()?;
        code_writer.write_push_pop(CommandType::Push, "that", 5)?;

        let mut asm_file_content = String::new();
        File::open(&test_file_name)?.read_to_string(&mut asm_file_content)?;

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
        assert_eq!(unindent(expect_asm), unindent(&asm_file_content));

        fs::remove_file(test_file_name)?;
        Ok(())
    }

    #[test]
    fn test_pop_command_when_static() -> Result<()> {
        let (mut code_writer, test_file_name) = get_code_writer()?;
        let segment = "static";
        let index = 10;
        code_writer.write_push_pop(CommandType::Pop, &segment, index)?;

        let mut asm_file_content = String::new();
        File::open(&test_file_name)?.read_to_string(&mut asm_file_content)?;

        let expect_asm = format!(r#"// pop
        @SP
        M=M-1
        A=M
        D=M

        // static {}
        @{}.{}
        M=D
        "#,
        index,
        Path::new(&test_file_name).file_stem().unwrap().to_string_lossy(),
        index,
        );
        assert_eq!(unindent(&expect_asm), unindent(&asm_file_content));

        fs::remove_file(test_file_name)?;
        Ok(())
    }

    #[test]
    fn test_pop_command() -> Result<()> {
        let (mut code_writer, test_file_name) = get_code_writer()?;
        code_writer.write_push_pop(CommandType::Pop, "local", 0)?;

        let mut asm_file_content = String::new();
        File::open(&test_file_name)?.read_to_string(&mut asm_file_content)?;

        let expect_asm = r#"// pop
        @SP
        M=M-1
        A=M
        D=M

        // local 0
        @LCL
        A=M+0
        M=D
        "#;
        assert_eq!(unindent(expect_asm), unindent(&asm_file_content));

        fs::remove_file(test_file_name)?;
        Ok(())
    }

    #[test]
    fn test_write_arithmetic() -> Result<()> {
        let (mut code_writer, test_file_name) = get_code_writer()?;
        code_writer.write_arithmetic("add",0)?;

        let mut asm_file_content = String::new();
        File::open(&test_file_name)?.read_to_string(&mut asm_file_content)?;

        let expect_asm = r#"// pop
        @SP
        M=M-1
        A=M
        D=M
        @R13
        M=D

        // pop
        @SP
        M=M-1
        A=M
        D=M
        @R13

        // add
        D=D+M

        // push
        @SP
        A=M
        M=D
        @SP
        M=M+1
        "#;
        assert_eq!(unindent(expect_asm), unindent(&asm_file_content));

        fs::remove_file(test_file_name)?;
        Ok(())
    }

    #[test]
    fn test_write_infinity_loop() -> Result<()> {
        let (mut code_writer, test_file_name) = get_code_writer()?;
        code_writer.write_infinity_loop()?;

        let mut asm_file_content = String::new();
        File::open(&test_file_name)?.read_to_string(&mut asm_file_content)?;

        let expect_asm = r#"
        (END)
        @END
        0;JMP
        "#;
        assert_eq!(unindent(expect_asm), unindent(&asm_file_content));

        fs::remove_file(test_file_name)?;
        Ok(())
    }
}
