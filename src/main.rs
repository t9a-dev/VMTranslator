use anyhow::Result;
use std::{fs::File, io::BufReader, path::{Path, PathBuf}};

const ASSEMBLY_FILE_EXTENSION: &str = "asm";

fn main() -> Result<()> {
    if let Err(e) = vm_translator(&parse_arg(std::env::args().collect())?) {
        eprintln!("{}", e);
        std::process::exit(1);
    }
    Ok(())
}

fn parse_arg(args: Vec<String>) -> Result<String> {
    let current_dir = "./".to_string();
    match args.get(1) {
        Some(arg) if arg.is_empty() => Ok(current_dir),
        Some(arg) => Ok(arg.to_string()),
        _ => Ok(current_dir),
    }
}

fn vm_translator(path_str: &str) -> Result<()> {
    let path = Path::new(path_str);
    let is_dir = path.is_dir();
    let mut vm_files: Vec<PathBuf> = Vec::new();
    // 引数で指定されたのがディレクトリであればvmファイルのパスを読み取る
    if is_dir {
        for entry in path.read_dir()? {
            if let Ok(entry) = entry {
                //　現在のディレクトリのファイルまで見る。再帰的にディレクトリに潜っていくことはしない。
                if entry.path().is_file() {
                    match entry.path().extension() {
                        Some(file_extension) if file_extension == "vm" => {
                            vm_files.push(entry.path());
                        }
                        _ => (),
                    }
                }
            }
        }
    } else if let Some(extension) = path.extension() {
        if extension == "vm" {
            vm_files.push(path.to_path_buf());
        } else {
            println!("un supported file: {:?}", path);
            return Ok(());
        }
    }

    let output_asm_file_name = if is_dir {
        path.file_name().unwrap().to_string_lossy().to_string()
    } else {
        vm_files
            .get(0)
            .unwrap()
            .to_path_buf()
            .file_stem()
            .unwrap()
            .to_string_lossy()
            .to_string()
    };
    let output_asm_file_path = path.parent().unwrap().join(format!(
        "{}.{}",
        output_asm_file_name, ASSEMBLY_FILE_EXTENSION
    ));

    let mut code_writer = code_writer::CodeWriter::new(&output_asm_file_path);
    vm_files.iter().try_for_each(|vm_file: &PathBuf| -> Result<()>{
        code_writer.set_filename(&vm_file)?;
        let mut parser = parser::Parser::new(BufReader::new(File::open(vm_file)?));
        while parser.has_more_lines()? {
            parser.advance()?;

            match parser.command_type()?.unwrap() {
                parser::CommandType::Arithmetic => {
                    code_writer.write_arithmetic(parser.arg1().unwrap().as_str())?;
                }
                parser::CommandType::Push | parser::CommandType::Pop => {
                    code_writer.write_push_pop(
                        parser.command_type()?.unwrap(),
                        parser.arg1().unwrap().as_str(),
                        parser.arg2()?.unwrap(),
                    )?;
                }
                parser::CommandType::Label => {
                    code_writer.write_label(&parser.arg1().unwrap())?;
                }
                parser::CommandType::Goto => {
                    code_writer.write_goto(&parser.arg1().unwrap())?;
                }
                parser::CommandType::If => {
                    code_writer.write_if(&parser.arg1().unwrap())?;
                }
                parser::CommandType::Function => {
                    let function_name = parser.arg1()?;
                    let n_vars = parser.arg2()?.unwrap();
                    code_writer.write_function(&function_name, n_vars)?;
                }
                parser::CommandType::Return => {
                    code_writer.write_return()?;
                }
                parser::CommandType::Call => {
                    let function_name = parser.arg1()?;
                    let n_args = parser.arg2()?.unwrap();
                    code_writer.write_call(&function_name, n_args)?;
                }
            }

            if !parser.has_more_lines()? {
                break;
            }
            code_writer.increment_uniq_index();
        }
        Ok(())
    })?;

    code_writer.close()?;
    println!("Translated: {}", &output_asm_file_path.to_string_lossy());

    Ok(())
}

#[cfg(test)]
mod tests {
    use anyhow::Result;

    use crate::{parse_arg, vm_translator};

    #[test]
    fn parse_test() -> Result<()> {
        let expect = "./";
        let args = vec!["".to_string(), "".to_string()];
        assert_eq!(parse_arg(args)?, expect.to_string());

        let expect = "test_vm_files/8/FunctionCalls/FibonacciElement";
        let args = vec!["".to_string(), expect.to_string()];
        assert_eq!(parse_arg(args)?, expect.to_string());

        Ok(())
    }

    #[test]
    fn run_translator() -> Result<()> {
        let args = vec![
            "".to_string(),
            "test_vm_files/8/FunctionCalls/FibonacciElement".to_string(),
        ];
        vm_translator(&parse_arg(args)?)?;

        Ok(())
    }
}
