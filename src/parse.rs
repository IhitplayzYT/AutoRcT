pub mod Parse{
    use std::collections::{HashMap, HashSet};
    use regex::Regex;

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
        // Remove comments and empty lines
        let mut file = file.split("\n").map(|x| x.trim()).filter(|x| !x.is_empty() || x.starts_with("//")).collect::<Vec<&str>>().join("\n");
        assert!(file.contains("@FMAP"));
        let mut in_str = false;

        let re = Regex::new(r"(?s)FUNC\s+([A-Za-z_][A-Za-z0-9_]*)\s*\(([^)]*)\)\s*\{(.*?)\}").unwrap();
        let mut varibs: Vec<HashMap<String,(String,bool)>> = Vec::new();
        varibs.push(HashMap::new());
        let lines = file.lines().peekable();
        let mut scope = 0_usize;


        // LOADS global variables at start
        for i in lines{
            if i.contains("{"){
                scope += 1;
            }
            if i.contains("}"){
                scope = scope.saturating_sub(1);
            }

            if scope > 0 || i.ends_with("}"){
                continue;
            }else{
                if i.contains(":=") {
                    let (k,v) = i.split_once(":=").unwrap();
                    varibs.last_mut().unwrap().insert(k.to_string(),(v.to_string(),false));
                }
            }
        }

        // COpy pastes all global varibs
        for scope_varibs in &varibs{
            for (k,(v,_)) in scope_varibs{
                file = file.replace(&format!("@{k}"), &v);
            }
        }

        varibs.clear();
        varibs.push(HashMap::new()); // Reset stack

        // Loads fxns
        for cap in re.captures_iter(&file) {
            let fn_name = &cap[1];
            let param_list = &cap[2];
            let fn_body = &cap[3];
            varibs.last_mut().unwrap().insert(format!("{fn_name}({param_list})"),(fn_body.to_string(),true));
        }


        // IGNORES are parsed
        if file.contains("@IGNORE"){
            let sec_st = file.find("@IGNORE").unwrap();
            let st = file[sec_st..].find("{").unwrap();
            let ed = file[sec_st+st..].find("}").unwrap();
            file[sec_st+st..(sec_st+st+ed)].split(|ch| ch == ',' || ch == '\n').for_each(|x| {
                let tr = x.trim();
                if !tr.is_empty(){
                    conf.ignore_files.insert(tr.to_string());
                }
            });
        
        file.drain(sec_st..sec_st+st+ed);
        }

        








        conf
    }





}