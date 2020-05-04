use ast::ExprBuilder;
use lexer::Lexer;
use parser::Parser;
use span::SourceFiles;
use std::path::Path;

fn run() -> bool {
    let path = Path::new("./depth_5.spd");

    let mut src_files = SourceFiles::new();
    let (_, file_name) = src_files.load_source_file(path);

    let src_file = src_files.get_by_name(&file_name);

    for _ in 0..650000 {
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

        let builder = ExprBuilder::new();
        let ast = {
            let mut parser = Parser::new(&builder, &tokens);
            match parser.parse_expr_eof() {
                Result::Err(err) => {
                    err.reportable().report(&src_files);
                    return false;
                }
                Result::Ok(expr) => expr,
            }
        };
    }

    true
}

fn main() {
    std::process::exit(match run() {
        true => 0,
        false => 1,
    })
}
