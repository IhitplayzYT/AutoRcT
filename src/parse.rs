pub mod Parse{
    use std::collections::{HashMap, HashSet};


    pub struct Config{
       pub check_cmds: HashMap<String,Cmd>,
       pub compile_cmnds: HashMap<String,Cmd>,
       pub ignore_files: HashSet<String>,    
    }

    impl Default for Config{
        fn default() -> Self {
            Self { check_cmds: HashMap::new(), compile_cmnds: HashMap::new(), ignore_files: HashSet::new() }
        }
    }

    impl Config{
        pub fn new() -> Self{
            Self::default()
        }
    }

    pub struct Cmd{
        pub path: String,
        pub cmd: String,
        pub cond_cmd: Option<String>
    }

    impl Default for Cmd{
        fn default() -> Self {
            Cmd { path: "".to_string(), cmd: "".to_string(), cond_cmd: None }
        }
    }

    impl Cmd{
        pub fn new() -> Self{
            Self::default()
        }
    }


    pub fn parse_conf(file: &str) -> Config{
        let mut conf = Config::new();
        let file = file.split("\n").map(|x| x.trim()).filter(|x| !x.is_empty()).collect::<Vec<&str>>().join("\n");
        
        



        conf
    }





}