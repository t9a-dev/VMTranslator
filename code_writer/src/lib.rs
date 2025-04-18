pub mod helper;

use std::{fs::File, io::Write, path::Path};

use anyhow::{Ok, Result};
use helper::arithmetic::ArithmeticCommandHelper;
use std::convert::AsRef;
use strum_macros::AsRefStr;
use unindent::{unindent, unindent_bytes};

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
    incremental_uniq_index: u16,
    // 無限ループで終了するようにENDラベルを必ず生成するのでVMコード内で記述されている場合に検知して重複を避ける
    has_end_label: bool,
}

impl CodeWriter {
    pub fn new(filename: &Path) -> Self {
        Self {
            assembly_file: Box::new(File::create(filename).unwrap()),
            filename: filename.file_stem().unwrap().to_string_lossy().to_string(),
            incremental_uniq_index: 0,
            has_end_label: false,
        }
    }

    pub fn write_arithmetic(&mut self, command: &str) -> Result<()> {
        let is_single_operand = command == "neg" || command == "not";
        let variable_register = VariableRegister::R13;

        self.write_code(unindent(
            format!(
                "{}
{}
{}
{}
{}",
                self.get_pop_code()?,
                self.get_load_register_code(variable_register)?,
                if !is_single_operand {
                    self.get_pop_code()?
                } else {
                    "".to_string()
                },
                ArithmeticCommandHelper::get_command(
                    command,
                    &variable_register,
                    self.incremental_uniq_index,
                )?,
                self.get_push_code()?,
            )
            .as_str(),
        ))?;

        Ok(())
    }

    pub fn write_push_pop(
        &mut self,
        command: CommandType,
        segment: &str,
        index: u16,
    ) -> Result<()> {
        self.write_code(self.get_segment_code(command, segment, index)?)?;
        Ok(())
    }

    pub fn write_label(&mut self, label: &str) -> Result<()> {
        if !self.has_end_label {
            if label == "END" {
                self.has_end_label = true;
            }
        }
        self.write_code(format!(
            "
({})
",
            label
        ))?;
        Ok(())
    }

    pub fn write_goto(&mut self, label: &str) -> Result<()> {
        self.write_code(self.get_goto_code(label))?;
        Ok(())
    }

    pub fn write_if(&mut self, label: &str) -> Result<()> {
        self.write_code(format!(
            "
{}
@{}
D;JGT
",
            self.get_pop_code()?,
            label,
        ))?;
        Ok(())
    }

    pub fn write_function(&mut self, function_name: &str, n_vars: u16) -> Result<()> {
        self.write_code(self.get_function_code(function_name, n_vars))?;
        Ok(())
    }

    pub fn write_call(&mut self, function_name: &str, n_args: u16) -> Result<()> {
        self.write_code(self.get_call_code(function_name, n_args))?;
        Ok(())
    }

    pub fn write_return(&self) -> Result<()> {
        Ok(())
    }

    pub fn close(mut self) -> Result<()> {
        self.write_code(self.get_infinity_loop_code()?)?;
        drop(self.assembly_file);
        Ok(())
    }

    pub fn increment_uniq_index(&mut self) {
        self.incremental_uniq_index += 1;
    }

    fn write_code(&mut self, code: String) -> Result<()> {
        self.assembly_file.write(&unindent_bytes(code.as_bytes()))?;
        Ok(())
    }

    fn get_segment_code(&self, command: CommandType, segment: &str, index: u16) -> Result<String> {
        let index_for_temp_segment = index + 5; //TEMPセグメントはRAM[5~12]固定
        let variable_register = VariableRegister::R13;
        let segment_symbol_asm = match segment {
            "local" => Some(format!("// local {}\n@LCL", index)),
            "argument" => Some(format!("// argument {}\n@ARG", index)),
            "this" => Some(format!("// this {}\n@THIS", index)),
            "that" => Some(format!("// that {}\n@THAT", index)),
            "temp" => Some(format!("// temp {}", index_for_temp_segment)),
            "constant" => Some(format!("// constant {}\n@{}\n", index, index)),
            "pointer" if index == 0 => Some(format!("// this {}\n@THIS", index)),
            "pointer" if index == 1 => Some(format!("// that {}\n@THAT", index)),
            "static" => Some(format!(
                "// static {}\n@{}.{}\n",
                index, self.filename, index
            )),
            _ => None,
        };

        let segment_code = match command {
            CommandType::Push => match segment {
                "constant" => {
                    format!(
                        "{}D=A\n{}",
                        segment_symbol_asm.unwrap(),
                        self.get_push_code()?
                    )
                }
                "temp" => {
                    format!(
                        "
{}
@{}
D=M
{}
",
                        segment_symbol_asm.unwrap(),
                        index_for_temp_segment,
                        self.get_push_code()?,
                    )
                }
                "pointer" | "static" => {
                    format!(
                        "
{}
D=M
{}
",
                        segment_symbol_asm.unwrap(),
                        self.get_push_code()?,
                    )
                }
                _ => {
                    format!(
                        "
@{}
D=A
{}
A=D+M
D=M
{}
",
                        index,
                        segment_symbol_asm.unwrap(),
                        self.get_push_code()?,
                    )
                }
            },
            CommandType::Pop => match segment {
                "static" => {
                    format!(
                        "
{}
{}
M=D
",
                        self.get_pop_code()?,
                        segment_symbol_asm.unwrap()
                    )
                }
                "temp" => {
                    format!(
                        "
{}
@{}
D=A
@{}
M=D
{}
@{}
A=M
M=D
",
                        segment_symbol_asm.unwrap(),
                        index_for_temp_segment,
                        &variable_register.as_ref(),
                        self.get_pop_code()?,
                        &variable_register.as_ref(),
                    )
                }
                "pointer" => {
                    format!(
                        "
{}
D=A
@{}
M=D
{}
@{}
A=M
M=D
",
                        segment_symbol_asm.unwrap(),
                        &variable_register.as_ref(),
                        self.get_pop_code()?,
                        &variable_register.as_ref(),
                    )
                }
                _ => {
                    format!(
                        "
@{}
D=A
{}
D=D+M
@{}
M=D
{}
@{}
A=M
M=D
",
                        index,
                        segment_symbol_asm.unwrap(),
                        &variable_register.as_ref(),
                        self.get_pop_code()?,
                        &variable_register.as_ref(),
                    )
                }
            },
            _ => panic!("get segment code failed"),
        };

        Ok(segment_code.to_string())
    }

    fn get_goto_code(&self, label: &str) -> String {
        format!(
            "
@{}
0;JMP
",
            label
        )
    }

    fn get_function_code(&self,function_name: &str,n_vars: u16) -> String{
        let mut buffer = String::new();
        buffer += &format!("
({})
",function_name);
        if n_vars == 0 {
           return buffer;     
        }

        let init_var_asm  = || -> String {
format!("
@0
D=A
{}
        ",self.get_push_code().unwrap())
        };

        for _ in 0..n_vars {
            buffer += &init_var_asm();
        }
        buffer
    }

    fn get_call_code(&self, function_name: &str, n_args: u16) -> String {
        let return_address_symbol = format!(
            "{}$ret.{}",
            function_name, self.incremental_uniq_index
        );
        let gen_push_segment_code = |symbol: &str| -> String {
            format!(
                "
@{}
D=A
{}
",
                symbol,
                self.get_push_code().unwrap()
            )
        };
        format!(
            "
// call function {}
{}
{}
{}
{}
{}

// ARG=SP-5-nArgs
@5
D=A
@{}
D=D-A
@SP
D=M-D
@ARG
M=D

// LCL=SP
@SP
D=M
@LCL
M=D

// goto function {}
{}

({})
",
            function_name,
            gen_push_segment_code(&return_address_symbol),
            gen_push_segment_code("LCL"),
            gen_push_segment_code("ARG"),
            gen_push_segment_code("THIS"),
            gen_push_segment_code("THAT"),
            n_args,
            function_name,
            self.get_goto_code(function_name),
            return_address_symbol,
        )
    }

    fn get_push_code(&self) -> Result<String> {
        Ok("
// push
@SP
A=M
M=D
@SP
M=M+1
"
        .to_string())
    }

    fn get_pop_code(&self) -> Result<String> {
        Ok("
// pop
@SP
M=M-1
A=M
D=M
"
        .to_string())
    }

    fn get_load_register_code(&self, register: VariableRegister) -> Result<String> {
        Ok(format!(
            "@{}
M=D",
            register.as_ref()
        ))
    }

    fn get_infinity_loop_code(&self) -> Result<String> {
        Ok(format!(
            "{}
@END
0;JMP
        ",
            if self.has_end_label { "" } else { "(END)" }
        )
        .to_string())
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

    fn normalize(s: &str) -> String {
        s.lines().map(str::trim).collect::<Vec<_>>().join("")
    }

    #[test]
    fn playground() -> Result<()> {
        assert_eq!("R13", VariableRegister::R13.as_ref());
        Ok(())
    }

    #[test]
    fn test_write_segment_when_constant() -> Result<()> {
        let (code_writer, test_file_name) = get_code_writer()?;
        let (segment, index) = ("constant", 10);
        let asm_file_content = code_writer.get_segment_code(CommandType::Push, &segment, index)?;

        let expect_asm = "// constant 10
        @10
        D=A
        // push
        @SP
        A=M
        M=D
        @SP
        M=M+1";
        assert_eq!(normalize(expect_asm), normalize(&asm_file_content));

        fs::remove_file(test_file_name)?;
        Ok(())
    }

    #[test]
    fn test_write_segment_when_push() -> Result<()> {
        let (code_writer, test_file_name) = get_code_writer()?;
        let (segment, index) = ("that", 5);
        let asm_file_content = code_writer.get_segment_code(CommandType::Push, &segment, index)?;

        let expect_asm = format!(
            "@{}
        D=A
        // that {}
        @THAT
        A=D+M
        D=M
        // push
        @SP
        A=M
        M=D
        @SP
        M=M+1",
            index, index,
        );
        assert_eq!(normalize(&expect_asm), normalize(&asm_file_content));

        fs::remove_file(test_file_name)?;
        Ok(())
    }

    #[test]
    fn test_push_command() -> Result<()> {
        let (mut code_writer, test_file_name) = get_code_writer()?;
        let (segment, index) = ("that", 5);
        code_writer.write_push_pop(CommandType::Push, &segment, index)?;

        let mut asm_file_content = String::new();
        File::open(&test_file_name)?.read_to_string(&mut asm_file_content)?;

        let expect_asm = format!(
            "@{}
        D=A
        // that {}
        @THAT
        A=D+M
        D=M
        // push
        @SP
        A=M
        M=D
        @SP
        M=M+1",
            index, index,
        );
        assert_eq!(normalize(&expect_asm), normalize(&asm_file_content));

        fs::remove_file(test_file_name)?;
        Ok(())
    }

    #[test]
    fn test_push_command_when_temp() -> Result<()> {
        let (mut code_writer, test_file_name) = get_code_writer()?;
        let (segment, index) = ("temp", 6);
        code_writer.write_push_pop(CommandType::Push, &segment, index)?;

        let mut asm_file_content = String::new();
        File::open(&test_file_name)?.read_to_string(&mut asm_file_content)?;

        let expect_asm = format!(
            "
        // temp {}
        @{}
        D=M
        // push
        @SP
        A=M
        M=D
        @SP
        M=M+1",
            index + 5,
            index + 5,
        );
        assert_eq!(normalize(&expect_asm), normalize(&asm_file_content));

        fs::remove_file(test_file_name)?;
        Ok(())
    }

    #[test]
    fn test_pop_command_when_static() -> Result<()> {
        let (mut code_writer, test_file_name) = get_code_writer()?;
        let (segment, index) = ("static", 10);
        code_writer.write_push_pop(CommandType::Pop, &segment, index)?;

        let mut asm_file_content = String::new();
        File::open(&test_file_name)?.read_to_string(&mut asm_file_content)?;

        let expect_asm = format!(
            "// pop
        @SP
        M=M-1
        A=M
        D=M
        // static {}
        @{}.{}
        M=D",
            index,
            Path::new(&test_file_name)
                .file_stem()
                .unwrap()
                .to_string_lossy(),
            index,
        );
        assert_eq!(normalize(&expect_asm), normalize(&asm_file_content));

        fs::remove_file(test_file_name)?;
        Ok(())
    }

    #[test]
    fn test_pop_command_when_temp() -> Result<()> {
        let (mut code_writer, test_file_name) = get_code_writer()?;
        let (segment, index) = ("temp", 6);
        code_writer.write_push_pop(CommandType::Pop, &segment, index)?;

        let mut asm_file_content = String::new();
        File::open(&test_file_name)?.read_to_string(&mut asm_file_content)?;

        let expect_asm = format!(
            "// temp {}
@{}
D=A

@{}
M=D

// pop
@SP
M=M-1
A=M
D=M

@R13
A=M
M=D
        ",
            index + 5,
            index + 5,
            VariableRegister::R13.as_ref(),
        );
        assert_eq!(normalize(&expect_asm), normalize(&asm_file_content));

        fs::remove_file(test_file_name)?;
        Ok(())
    }

    #[test]
    fn test_pop_command() -> Result<()> {
        let (mut code_writer, test_file_name) = get_code_writer()?;
        let (segment, index) = ("local", 6);
        code_writer.write_push_pop(CommandType::Pop, &segment, index)?;

        let mut asm_file_content = String::new();
        File::open(&test_file_name)?.read_to_string(&mut asm_file_content)?;

        let expect_asm = format!(
            "@{}
        D=A
        // local {}
        @LCL
        D=D+M
        @R13
        M=D
        // pop
        @SP
        M=M-1
        A=M
        D=M
        @R13
        A=M
        M=D
        ",
            index, index,
        );
        assert_eq!(normalize(&expect_asm), normalize(&asm_file_content));

        fs::remove_file(test_file_name)?;
        Ok(())
    }

    #[test]
    fn test_write_arithmetic() -> Result<()> {
        let (mut code_writer, test_file_name) = get_code_writer()?;
        code_writer.write_arithmetic("add")?;

        let mut asm_file_content = String::new();
        File::open(&test_file_name)?.read_to_string(&mut asm_file_content)?;

        let expect_asm = "// pop
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
        M=M+1";
        assert_eq!(normalize(expect_asm), normalize(&asm_file_content));

        fs::remove_file(test_file_name)?;
        Ok(())
    }

    #[test]
    fn test_write_arithmetic_when_sub() -> Result<()> {
        let (mut code_writer, test_file_name) = get_code_writer()?;
        code_writer.write_arithmetic("sub")?;

        let mut asm_file_content = String::new();
        File::open(&test_file_name)?.read_to_string(&mut asm_file_content)?;

        let expect_asm = "// pop
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
        // sub
        D=D-M
        // push
        @SP
        A=M
        M=D
        @SP
        M=M+1";
        assert_eq!(normalize(expect_asm), normalize(&asm_file_content));

        fs::remove_file(test_file_name)?;
        Ok(())
    }

    #[test]
    fn test_write_call_code() -> Result<()> {
        let (mut code_writer, test_file_name) = get_code_writer()?;
        let (function_name, n_args) = ("SimpleFunction.test", 2);
        code_writer.write_call(function_name,n_args)?;
        let expect = format!("
// call function SimpleFunction.test
@{}$ret.0
D=A
// push
@SP
A=M
M=D
@SP
M=M+1

@LCL
D=A
// push
@SP
A=M
M=D
@SP
M=M+1

@ARG
D=A
// push
@SP
A=M
M=D
@SP
M=M+1

@THIS
D=A
// push
@SP
A=M
M=D
@SP
M=M+1

@THAT
D=A
// push
@SP
A=M
M=D
@SP
M=M+1

// ARG=SP-5-nArgs
@5
D=A
@{}
D=D-A
@SP
D=M-D
@ARG
M=D

// LCL=SP
@SP
D=M
@LCL
M=D

// goto function {}
@{}
0;JMP

({}$ret.0)
",
function_name,
n_args,
function_name,
function_name,
function_name,
);
        let mut actual = String::new();
        File::open(&test_file_name)?.read_to_string(&mut actual)?;

        assert_eq!(normalize(&expect),normalize(&actual));

        fs::remove_file(test_file_name)?;
        Ok(())
    }

    #[test]
    fn test_write_function_code() -> Result<()> {
        let (mut code_writer, test_file_name) = get_code_writer()?;
        let (function_name, n_args) = ("SimpleFunction.test", 1);
        code_writer.write_function(function_name,n_args)?;
        
        let expect = format!("
({})
@0
D=A
// push
@SP
A=M
M=D
@SP
M=M+1
",
function_name,
);
        let mut actual = String::new();
        File::open(&test_file_name)?.read_to_string(&mut actual)?;

        assert_eq!(normalize(&expect),normalize(&actual));

        fs::remove_file(test_file_name)?;
        Ok(())
    }

    #[test]
    fn test_write_function_code_when_args_zero() -> Result<()> {
        let (mut code_writer, test_file_name) = get_code_writer()?;
        let (function_name, n_args) = ("SimpleFunction.test", 0);
        code_writer.write_function(function_name,n_args)?;
        
        let expect = format!("
({})
",
function_name,
);
        let mut actual = String::new();
        File::open(&test_file_name)?.read_to_string(&mut actual)?;

        assert_eq!(normalize(&expect),normalize(&actual));

        fs::remove_file(test_file_name)?;
        Ok(())
    }

    #[test]
    fn test_write_infinity_loop() -> Result<()> {
        let (code_writer, test_file_name) = get_code_writer()?;
        let asm_file_content = code_writer.get_infinity_loop_code()?;

        let expect_asm = "
        (END)
        @END
        0;JMP
        ";
        assert_eq!(normalize(expect_asm), normalize(&asm_file_content));

        fs::remove_file(test_file_name)?;
        Ok(())
    }
}
