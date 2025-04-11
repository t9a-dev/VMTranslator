use anyhow::Result;
use clap::Parser;
use std::path::Path;

const ASSEMBLY_FILE_EXTENSION: &str = "asm";

#[derive(Debug, Parser)]
#[command(author, version, about)]
pub struct Arg {
    // HackAsembler File Path
    #[arg(value_name = "FILE_NAME.vm", short)]
    file: String,
}

fn main() -> Result<()> {
    if let Err(e) = vm_translator(&Arg::parse()) {
        eprintln!("{}", e);
        std::process::exit(1);
    }
    Ok(())
}

fn vm_translator(config: &Arg) -> Result<String> {
    let vm_file_path = Path::new(&config.file);
    let mut parser = parser::Parser::new(&config.file);
    let asm_file_path = vm_file_path.parent().unwrap().join(format!(
        "{}.{}",
        vm_file_path.file_stem().unwrap().to_string_lossy(),
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
            parser::CommandType::Label => todo!(),
            parser::CommandType::Goto => todo!(),
            parser::CommandType::If => todo!(),
            parser::CommandType::Function => todo!(),
            parser::CommandType::Return => todo!(),
            parser::CommandType::Call => todo!(),
        }

        if !parser.has_more_lines()? {
            break;
        }
    }

    code_writer.close()?;
    println!("Translated: {}", &asm_file_path.to_string_lossy());

    Ok(asm_file_path.to_string_lossy().to_string())
}

#[cfg(test)]
mod tests {
    use std::{fs, io::Write, path::Path, process::Command};

    use anyhow::Result;
    use rand::distr::{Alphanumeric, SampleString};

    use crate::vm_translator;

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
    fn run_translator() -> Result<()> {
        let config = super::Arg {
            file: "test_vm_files/StackTest.vm".to_string(),
        };
        vm_translator(&config)?;

        Ok(())
    }
}
