use std::{env::args, process::exit};

use abnf2lalrpop::parser;
use getopts_macro::getopts_options;

fn main() {
    let options = getopts_options! {
        -h, --help          "show help";
        -v, --version       "show help";
    };
    let matches = match options.parse(args().skip(1)) {
        Ok(it) => it,
        Err(e) => {
            eprintln!("{e}");
            exit(2)
        },
    };
    if let Some(get) = matches.free.get(1) {
        eprintln!("Extra arg: {get:?}");
        exit(2)
    }
    if matches.opt_present("help") {
        println!("{}", options.short_usage(env!("CARGO_BIN_NAME")).trim());
        return;
    }
    if matches.opt_present("version") {
        println!("{}", env!("CARGO_PKG_VERSION"));
        return;
    }

    if let Some(first) = matches.free.first() {
        let src = match fs_err::read_to_string(first) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("{e}");
                exit(2)
            },
        };
        gen_lalrpop(&src);
    } else {
        let src = std::io::read_to_string(std::io::stdin().lock()).unwrap();
        gen_lalrpop(&src);
    }
}

fn gen_lalrpop(src: &str) {
    let defs = match parser::defs(src) {
        Ok(it) => it,
        Err(e) => {
            eprintln!("{e}");
            exit(1)
        },
    };
    println!("grammar;");
    for def in defs {
        println!("pub {def}")
    }
}
