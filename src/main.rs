use std::{error::Error, thread, time::Duration};

use crate::{daemon::daemon::InotifyMonitor, helper::Helper::CLI, parse::Parse::{Config, Exec::Cmd, Properties, parse_conf}};
mod helper;
mod parse;
mod daemon;
mod fpath;

fn main() -> Result<(),Box<dyn Error>>{
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
    //let conf = parse_conf(&fl);
    let mut conf = Config::new();
    let mut prop = Properties::default();
    prop.build = Some(Cmd("echo \"Building\"".to_string()));
    prop.clean = Some(Cmd("echo \"Cleaning\"".to_string()));
    prop.run = Some(Cmd("echo \"Running\"".to_string()));


    conf.exec_props.insert((".rs".to_string(),"~/AutoRcT/src".to_string()),prop);

    let orchestrator = InotifyMonitor::new(conf)?;
    let hndle = orchestrator.run();
    loop{
        thread::sleep(Duration::from_secs(1));    
    }
}
