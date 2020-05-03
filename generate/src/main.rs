mod lib;

use lib::Generator;
use pretty::pretty_expr;
use std::fs::File;
use std::io::Write;
use std::str::FromStr;

fn run() -> std::io::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let gen = Generator::new();
    let expr = gen.gen_expr(u32::from_str(&args[1]).unwrap());
    let mut file = File::create(&args[2])?;
    write!(file, "{}", pretty_expr(expr))
}

fn main() {
    run().unwrap()
}
