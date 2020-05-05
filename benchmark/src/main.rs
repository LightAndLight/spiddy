use ast::de_bruijn;
use ast::syntax;
use eval::eval_loop;
use eval::heap::Heap;
use eval::stack::Stack;
use lexer::Lexer;
use parser::Parser;
use span::SourceFiles;
use std::path::Path;

fn build_eval_expr<'builder, 'value>(
    builder: &'builder de_bruijn::ExprBuilder<'value>,
) -> de_bruijn::ExprRef<'value>
where
    'builder: 'value,
{
    let nil =
        // \n ->
        builder.mk_lam(
            // \c ->
            builder.mk_lam(
                // n
                builder.mk_var(1),
            ),
        );
    let cons =
        // \a ->
        builder.mk_lam(
            // \b ->
            builder.mk_lam(
                // \n ->
                builder.mk_lam(
                    // \c ->
                    builder.mk_lam(
                        builder.mk_app(
                            // c a
                            builder.mk_app(builder.mk_var(0), builder.mk_var(3)),
                            builder.mk_app(
                                // b n
                                builder.mk_app(builder.mk_var(2), builder.mk_var(1)),
                                // c
                                builder.mk_var(0)
                            )
                        ),
                    ),
                ),
            ),
        );
    let zero_to_5 = builder.mk_app(
        // cons 0
        builder.mk_app(cons, builder.mk_u64(0)),
        builder.mk_app(
            // cons 1
            builder.mk_app(cons, builder.mk_u64(1)),
            builder.mk_app(
                // cons 2
                builder.mk_app(cons, builder.mk_u64(2)),
                builder.mk_app(
                    // cons 3
                    builder.mk_app(cons, builder.mk_u64(3)),
                    builder.mk_app(
                        // cons 4
                        builder.mk_app(cons, builder.mk_u64(4)),
                        builder.mk_app(
                            // cons 5
                            builder.mk_app(cons, builder.mk_u64(5)),
                            //nil
                            nil,
                        ),
                    ),
                ),
            ),
        ),
    );
    builder.mk_app(
        // zero_to_5 0
        builder.mk_app(zero_to_5, builder.mk_u64(0)),
        // \a ->
        builder.mk_lam(
            // \b ->
            builder.mk_lam(
                // a + b
                builder.mk_addu64(builder.mk_var(1), builder.mk_var(0)),
            ),
        ),
    )
}

fn run() -> bool {
    let args: Vec<String> = std::env::args().into_iter().collect();
    match args[1].as_str() {
        /*
        "eval" => {
            let builder = de_bruijn::ExprBuilder::new();
            let expr = build_eval_expr(&builder);
            for _ in 0..450000 {
                let heap = Heap::with_capacity(1024);
                let _ = eval(&heap, &Vec::new(), expr);
            }
        }
        */
        "eval_loop" => {
            let builder = de_bruijn::ExprBuilder::new();
            let expr = build_eval_expr(&builder);
            for _ in 0..450000 {
                let heap = Heap::with_capacity(1024);
                let _ = eval_loop(&heap, expr);
            }
        }
        "parse" => {
            let path = Path::new("./depth_5.spd");

            let mut src_files = SourceFiles::new();
            let (_, file_name) = src_files.load_source_file(path);

            let src_file = src_files.get_by_name(&file_name);

            for _ in 0..950000 {
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

                let builder = syntax::ExprBuilder::new();
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
        }
        arg => panic!("Unexpected command line argument {:?}", arg),
    }

    true
}

fn main() {
    std::process::exit(match run() {
        true => 0,
        false => 1,
    })
}
