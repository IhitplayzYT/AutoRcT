pub mod Helper{
    use std::{path::PathBuf, process::exit};



    const DBG_STR: &str = "";
    const OK:i32 = 0;
    const ERR:i32 = -1;


    #[derive(Debug,Clone)]
    pub struct CLI{
        pub dbg: bool,
        pub root: PathBuf,
    }


    pub fn Help(){
        println!("{DBG_STR}");
        exit(OK);
    }


    impl CLI{
        pub fn new() -> Self{
            Self {dbg: false, root: std::env::current_dir().unwrap()}
        }

        pub fn Parse_Args(&mut self){
            let args: Vec<String> = std::env::args().skip(1).collect();
            for i in &args{
                if i == "-d" || i == "--debug" || i == " --DEBUG" || i == "-D"{
                    self.dbg = true;
                } else if i == "-h" || i == "--help" || i == " --HELP" || i == "-H"{
                    Help();
                } else if i.starts_with("--root="){
                    self.root = PathBuf::from(&i[i.find("=").unwrap()+1..]);
                }else{
                    Help();
                }
           } 


        }



    }


    





}