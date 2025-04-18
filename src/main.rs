use anyhow::Result;
use std::path::Path;

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
    if path.is_dir() {
        for entry in path.read_dir()? {
            if let Ok(entry) = entry {
                //　現在のディレクトリのファイルまで見る。再帰的にディレクトリに潜っていくことはしない。
                if entry.path().is_file() {
                    match entry.path().extension() {
                        Some(file_extension) if file_extension == "vm" => {
                            vm_translator(&entry.path().to_string_lossy().to_string().clone())?;
                        }
                        _ => (),
                    }
                }
            }
        }
        return Ok(());
    }
    if let Some(extension) = path.extension() {
        if extension != "vm" {
            println!("un supported file: {:?}", path);
            return Ok(());
        }
    }

    let mut parser = parser::Parser::new(path.to_str().unwrap());
    let asm_file_path = path.parent().unwrap().join(format!(
        "{}.{}",
        path.file_stem().unwrap().to_string_lossy(),
        ASSEMBLY_FILE_EXTENSION
    ));
    let mut code_writer = code_writer::CodeWriter::new(&asm_file_path);

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

    code_writer.close()?;
    println!("Translated: {}", &asm_file_path.to_string_lossy());

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::{fs, io::Write, path::Path};

    use anyhow::Result;
    use rand::distr::{Alphanumeric, SampleString};

    use crate::{parse_arg, vm_translator};

    fn create_test_file(file_content: &str) -> String {
        let filename = Alphanumeric.sample_string(&mut rand::rng(), 5);
        //bacon testでファイル変更検知が発生しないようにtargetディレクトリにテストファイルを作成する。
        let _ = fs::create_dir_all("../target/test/data");
        let file_path = Path::new("../target/test/data").join(&filename);
        let mut file = fs::File::create(&file_path).unwrap();
        file.write(file_content.as_bytes()).unwrap();

        file_path.to_string_lossy().to_string()
    }

    #[test]
    fn parse_test() -> Result<()> {
        let expect = "./";
        let args = vec!["".to_string(), "".to_string()];
        assert_eq!(parse_arg(args)?, expect.to_string());

        let expect = "test_vm_files/8/FunctionCalls/SimpleFunction/SimpleFunction.vm";
        let args = vec!["".to_string(), expect.to_string()];
        assert_eq!(parse_arg(args)?, expect.to_string());

        Ok(())
    }

    #[test]
    fn run_translator() -> Result<()> {
        let args = vec![
            "".to_string(),
            "test_vm_files/8/FunctionCalls/SimpleFunction/SimpleFunction.vm".to_string(),
        ];
        vm_translator(&parse_arg(args)?)?;

        Ok(())
    }
}
