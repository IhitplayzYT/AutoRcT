pub mod Parse{
    use std::{collections::{HashMap, HashSet}, fmt::Display, num::NonZero, process::{Command, ExitStatus}};
    use regex::Regex;

    pub struct Properties{
        pub build: Option<String>,
        pub run: Option<String>,
        pub clean: Option<String>,
        pub path: Option<String>,
        pub monitor_dir: String,
        pub events: u8,
        pub cond: Option<String>
    }

    impl Default for Properties{
        fn default() -> Self {
            Self { build: None, run: None, clean: None, path: None,events: u8::MAX, monitor_dir:".".to_string(),cond:None}
        }
    }

    pub struct Config {
        // (fpath,ext) => Properties 
       pub exec_props: HashMap<(String,String),Properties>,
       pub ignore_files: HashSet<String>,    
    }

    impl Default for Config{
        fn default() -> Self {
            Self { exec_props: HashMap::new(), ignore_files: HashSet::new()}
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
        let mut varibs: Vec<HashMap<String,(String,Option<String>)>> = Vec::new();
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
                    let (k,mut v)  = (k.trim(),v.trim().to_string());
                    if v.starts_with("CMD"){
                        let out = Command::new(&v[4..]).output().expect("Stuck and Terminated");
                        v = if out.status.success(){String::from_utf8(out.stdout).unwrap()} else{String::from_utf8(out.stderr).unwrap()};
                    }
                    varibs.last_mut().unwrap().insert(k.to_string(),(v,None));
                }

            }
        }

        // Copy pastes all global varibs
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
            varibs.last_mut().unwrap().insert(format!("{fn_name}"),(fn_body.to_string(),Some(param_list.to_string())));
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


        let sec_st = file.find("@FMAP").unwrap();
        let st = file[sec_st..].find("{").unwrap();
        let ed = file[sec_st+st..].find("}").unwrap();
        file[sec_st+st..sec_st+st+ed].lines().for_each(|line|{
            let (ext,prop) = line.split_once("->").unwrap();
            let (ext,prop) = (ext.trim().strip_suffix("\'").unwrap().strip_prefix("\'").unwrap().strip_suffix("\"").unwrap().strip_prefix("\"").unwrap().to_string(),prop.trim().strip_prefix("[").unwrap().strip_suffix("]").unwrap());
            let v_prop = prop.split(",").collect::<Vec<&str>>();
            assert!(v_prop.len() == 6);
            let mut prop = Properties::default();

            // We need the inspect route 
            assert!(!param_is_empty(v_prop[0]));

            if !v_prop[1].is_empty(){
                prop.cond = Some(v_prop[1].to_string());
            }
            
            if !v_prop[2].is_empty(){
                prop.build = Some(v_prop[1].to_string());
            }

            if !v_prop[3].is_empty(){
                prop.run = Some(v_prop[1].to_string());
            }

            if !v_prop[4].is_empty(){
                prop.clean = Some(v_prop[1].to_string());
            }

            if !v_prop[5].is_empty(){
                prop.events = parse_event(v_prop[5]);
            }

            if !v_prop[6].is_empty(){
                prop.monitor_dir = v_prop[6].to_string();
            }
            conf.exec_props.insert((ext,v_prop[0].to_string()), prop);
        });
        
        conf
    }


    fn param_is_empty(a: &str) -> bool{
        (a == "\"\"") || a == "\'\'" || a.is_empty()
    }

    fn parse_event(s: &str) -> u8{
        let mut ret = 0;
        let s = s.to_lowercase().chars().collect::<HashSet<char>>();
        for (i,ch) in "rwocdnt".chars().enumerate(){
            if s.contains(&ch){
                ret |= 1 << i;
            }
        }
        ret
    }


    pub enum t_Return{
        Bool(bool),
        Int(isize),
        Float(f64),
        Str(String),
        Null
    }

    impl Display for t_Return{
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f,"{}", match self{
                    t_Return::Bool(k) => {
                        if *k{
                            "true".to_string()
                        }else{
                            "false".to_string()
                        }
                    },
                    t_Return::Int(k) => {format!("{k}")},
                    t_Return::Float(k) => {format!("{k}")},
                    t_Return::Str(k) => k.to_string(),
                    t_Return::Null => "".to_string(),
                })
        }
    
    }




//    RET := CMD [[ -e @a]]
//    IF RET < 0
//        CMD echo "Echo of param failed"
//    ELSE
//        CMD cd @a
//    FI 
//
//    RETURN (TRUE)

    pub fn exec_fxn(params: HashMap<&str,&str>,mut body: &str,stack: &mut Vec<HashMap<String,(String,Option<String>)>>) -> t_Return{
        stack.push(HashMap::new());
        let mut body = body.to_string();
        // First Eval varibs and nexted fxn_calls
        let re = Regex::new(r"(?s)FUNC\s+([A-Za-z_][A-Za-z0-9_]*)\s*\(([^)]*)\)\s*\{(.*?)\}").unwrap();
        for line in body.lines().map(|x| x.trim()){
            if line.contains(":="){
                let (k,v) = line.split_once(":=").unwrap();
                let (k,mut v) = (k.trim(),v.trim().to_string());
                if v.starts_with("@"){
                    let st = k.find("(").unwrap();
                    let params = &k[st+1..k.rfind(")").unwrap()];
                    let fxn_name = &k[..st];
                    // Search stack for the fxn_name
                    let mut fxn_body = None;
                    let mut p_list = "".to_string();
                    for v in stack.iter(){
                        if let Some(z) = v.get(fxn_name){
                            fxn_body = Some(&z.0[..]);
                            p_list = z.1.clone().unwrap_or_default();
                            break;
                        }
                    }
                    if let Some(f_body) = fxn_body{
                        let p_map: HashMap<&str,&str> = p_list.split(",").map(|x| x.trim()).zip(params.split(",").map(|x| x.trim())).collect();
                        v = format!("{}",exec_fxn(p_map, &body, stack));
                    }else{
                        panic!("Undefined Fxn call used @{fxn_name}");
                    }
                }
                stack.last_mut().unwrap().insert(k.to_string(), (v,None));
            }
        }
        
        for layer in stack.iter(){
            for (k,(v,p_l)) in layer{
                if p_l.is_none(){
                    body = body.replace(&("@".to_string()+k), v);
                }else{
                    let mut st = 0;
                    while let Some(idx) = body[st..].find(&format!("@{k}")){
                        st += idx;
                        let ed = body[st..].find(")").unwrap();
                        let l = k.len()+1;
                        let p_params = &body[st+l+1..st+ed];
                        let p_l = &p_l.clone().unwrap_or_default()[..];

                        let p_map: HashMap<&str,&str> = p_l.split(",").map(|x| x.trim()).zip(p_params.split(",").map(|x| x.trim())).collect();
                        let mut st_cl = stack.clone();
                        body = body.replace(&body[st..st+ed],&format!("{}",exec_fxn(p_map, &body, &mut st_cl)));
                    }

                }                
            }
        }



        for line in body.lines().map(|x| x.trim()){
            if line.starts_with("IF") || line.starts_with("ELIF"){
            }
        }



        t_Return::Null
    }


    fn eval_half(half: &str) -> t_Return{
        let half = half.trim();
        


    }

    //  "s1" == "s2"
    fn eval_cond(cond: &str) -> bool{



        let cond = cond.trim();
        if let Some((lhs,rhs)) = cond.split_once("=="){
            return eval_cond(lhs) == eval_cond(rhs);
        }

        if let Some((lhs,rhs)) = cond.split_once("!="){
            return eval_cond(lhs) != eval_cond(rhs);
        }

        if let Some((lhs,rhs)) = cond.split_once(">"){
            return eval_cond(lhs) > eval_cond(rhs);
        }

        if let Some((lhs,rhs)) = cond.split_once("<"){
            return eval_cond(lhs) < eval_cond(rhs);
        }

        if let Some((lhs,rhs)) = cond.split_once("<="){
            return eval_cond(lhs) <= eval_cond(rhs);
        }

        if let Some((lhs,rhs)) = cond.split_once(">="){
            return eval_cond(lhs) >= eval_cond(rhs);
        }
        let mut c = 0;

        if cond.bytes().all(|x| {
            if x >= b'0'  && x <= b'9'{
                true
            }else{
                if x == b'.' && c == 0{
                    c += 1;
                    true
                }else{
                    false
                }
            }
        }){
            if c == 1{
                let num = cond.parse::<f64>().expect("NOT POSSIBLE");
                if num > 0.0{
                    return true
                }else{
                    return false
                }
            }else{
                let num = cond.parse::<isize>().expect("NOT POSSIBLE");
                if num > 0{
                    return true
                }else{
                    return false
                }
            }
        } else{
            let ret = match cond{
                "true" => Some(true),
                "false" => Some(false),
                _ => None
            };
            
            if let Some(z) = ret{
                return z;
            }else{
                if matches!(cond,"\"\""|"\'\'"|""){
                    return false
                }else{
                    return true
                }

            
            }


        }

        
        

        false
    }


}