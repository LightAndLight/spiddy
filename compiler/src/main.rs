use lexer::Lexer;
use parser::Parser;
use span::SourceFiles;
use std::path::Path;

fn run() -> bool {
    let args: Vec<String> = std::env::args().collect();
    let path = Path::new(&args[1]);

    let mut src_files = SourceFiles::new();
    let (_, file_name) = src_files.load_source_file(path);

    let src_file = src_files.get_by_name(&file_name);

    let tokens = {
        let lexer = Lexer::from_source_file(src_file);
        match lexer.tokenize() {
            Result::Err(err) => {
                err.reportable().report(&src_files);
                return false;
            }
            Result::Ok(tokens) => tokens,
        }
    };

    let ast = {
        let mut parser = Parser::new(&tokens);
        match parser.parse_expr_eof() {
            Result::Err(err) => {
                err.reportable().report(&src_files);
                return false;
            }
            Result::Ok(expr) => expr,
        }
    };

    // println!("{:?}", ast);

    true
}

fn main() {
    std::process::exit(match run() {
        true => 0,
        false => 1,
    })
}
