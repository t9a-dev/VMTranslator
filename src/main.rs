use anyhow::Result;
use clap::Parser;
use std::fs::File;
use std::io::Write;
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
    let code_writer = code_writer::CodeWriter::new(&asm_file_path);

    while parser.has_more_lines()? {
        parser.advance()?;

        match parser.command_type()?.unwrap(){
            parser::CommandType::Arithmetic => todo!(),
            parser::CommandType::Push => todo!(),
            parser::CommandType::Pop => todo!(),
            parser::CommandType::Label => todo!(),
            parser::CommandType::Goto => todo!(),
            parser::CommandType::If => todo!(),
            parser::CommandType::Function => todo!(),
            parser::CommandType::Return => todo!(),
            parser::CommandType::Call => todo!(),
        }

        if !parser.has_more_lines()?{
            break;
        }
    }

    code_writer.close()?;
    println!("Translated: {}", &asm_file_path.to_string_lossy());

    Ok(asm_file_path.to_string_lossy().to_string())
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        io::{Read, Write},
        path::Path,
    };

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
    fn playground() {
        let path = Path::new("/a/b/c.txt");
        assert_eq!(path.parent(), Some(Path::new("/a/b/")));
    }
}
