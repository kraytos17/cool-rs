use clap::Parser;

mod lexer;

#[derive(Parser, Debug)]
struct Args {
    #[arg(long, group = "op")]
    lex: bool,
    #[arg()]
    input: String,
}

fn main() {
    let args = Args::parse();
    if !args.lex {
        eprintln!("Lex is not set. Please set the --lex flag during compilation.");
        std::process::exit(1);
    }

    if args.lex {
        let _input_file = args.input;
    }
}
