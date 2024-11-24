use clap::Parser;

mod standard_brainfuck;
mod ezfuck;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    path: Option<String>,
}

fn interpret_string(code: &str) -> () {
    let instructions = ezfuck::parser::parser::compile_to_intermediate(code);
    ezfuck::interpreter::interpreter::interpret_with_std_io(&instructions);
}

fn main() {
    let args = Args::parse();

    match args.path {
        Some(path) => {
            match std::fs::read_to_string(path) {
                Ok(code) => {
                    interpret_string(code.as_str());
                }
                Err(err) => {
                    eprintln!("Could not read file: {err}");
                }
            }
        }
        None => {
            println!("REPL functionality is not implemented yet. Please pass a path to run code.");
        }
    }
}
