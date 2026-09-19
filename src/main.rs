use crate::{helper::Helper::CLI, parse::Parse::parse_conf};

mod helper;
mod parse;

fn main() {
    let mut clargs = CLI::new();
    clargs.Parse_Args();

    if clargs.dbg{
        println!("{clargs:?}");
    }
    let conf_p = clargs.root.join(".RCTconf");
    if !clargs.root.exists() || !conf_p.exists(){
        panic!("Root dir or CONF file doesnt exist")
    }

    let fl = std::fs::read_to_string(conf_p).unwrap();
    let conf = parse_conf(&fl);

    




}
