pub mod Parse{
    use std::{collections::{HashMap, HashSet}, fmt::Display, num::NonZero, process::{Command, ExitStatus}};
    use regex::Regex;
use serde::{Deserialize, Serialize};

    #[derive(Debug,Serialize,Deserialize)]
    pub struct Properties{
        pub build: Option<Exec>,
        pub run: Option<Exec>,
        pub clean: Option<Exec>,
        pub path: Option<Exec>,
        pub monitor_dir: String,
        pub events: u16,
        pub cond: Option<Exec>,
        pub stack: Vec<HashMap<String,(String,Option<String>)>>
    }

    #[derive(Debug,Serialize,Deserialize)]
    pub enum Exec{
        Cmd(String),
        Fn(String,HashMap<String,String>)
    } 

    impl Default for Properties{
        fn default() -> Self {
            Self { build: None, run: None, clean: None, path: None,events:((u8::MAX as u16) << 1) | 1 , monitor_dir:".".to_string(),cond:None,stack:vec![]}
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

        // Reset stack
        varibs.clear();
        varibs.push(HashMap::new());

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
            prop.stack =  varibs.clone();


            // COND
            if !v_prop[1].is_empty(){
                if v_prop[1].starts_with("@") {
                    let st = v_prop[1].find("(").unwrap();
                    let ed = v_prop[1].find(")").unwrap();
                    let mut brk = false;
                    for layer in varibs.iter(){
                        for (k,(fn_body,p_l)) in layer{
                            if let Some(p_sig) = p_l{  
                                let p_map: HashMap<String,String> = p_sig.split(",").map(| x| str::trim(x).to_string()).zip(v_prop[1][st+1..ed].split(",").map(|x| str::trim(x).to_string())).collect();
                                prop.cond = Some(Exec::Fn(fn_body.to_string(), p_map));
                                brk = true;
                                break;
                            }
                            if brk{
                                break;
                            }
                        }
                    }
                } else {
                    prop.cond = Some(Exec::Cmd(v_prop[1].to_string()))
                }
            }
            
            // BUILD
            if !v_prop[2].is_empty(){
                if v_prop[2].starts_with("@") {
                    let st = v_prop[2].find("(").unwrap();
                    let ed = v_prop[2].find(")").unwrap();
                    let mut brk = false;
                    for layer in varibs.iter(){
                        for (k,(fn_body,p_l)) in layer{
                            if let Some(p_sig) = p_l{  
                                let p_map: HashMap<String,String> = p_sig.split(",").map(| x| str::trim(x).to_string()).zip(v_prop[1][st+1..ed].split(",").map(|x| str::trim(x).to_string())).collect();
                                prop.build = Some(Exec::Fn(fn_body.to_string(), p_map));
                                brk = true;
                                break;
                            }
                            if brk{
                                break;
                            }
                        }
                    }
                } else {
                    prop.build = Some(Exec::Cmd(v_prop[2].to_string()))
                }
            }

            // RUN
            if !v_prop[3].is_empty(){
                if v_prop[3].starts_with("@") {
                    let st = v_prop[3].find("(").unwrap();
                    let ed = v_prop[3].find(")").unwrap();
                    let mut brk = false;
                    for layer in varibs.iter(){
                        for (k,(fn_body,p_l)) in layer{
                            if let Some(p_sig) = p_l{  
                                let p_map: HashMap<String,String> = p_sig.split(",").map(| x| str::trim(x).to_string()).zip(v_prop[1][st+1..ed].split(",").map(|x| str::trim(x).to_string())).collect();
                                prop.run = Some(Exec::Fn(fn_body.to_string(), p_map));
                                brk = true;
                                break;
                            }
                            if brk{
                                break;
                            }
                        }
                    }
                } else {
                    prop.run = Some(Exec::Cmd(v_prop[3].to_string()))
                }
            }

            // CLEAN
            if !v_prop[4].is_empty(){
                if v_prop[4].starts_with("@") {
                    let st = v_prop[4].find("(").unwrap();
                    let ed = v_prop[4].find(")").unwrap();
                    let mut brk = false;
                    for layer in varibs.iter(){
                        for (k,(fn_body,p_l)) in layer{
                            if let Some(p_sig) = p_l{  
                                let p_map: HashMap<String,String> = p_sig.split(",").map(| x| str::trim(x).to_string()).zip(v_prop[1][st+1..ed].split(",").map(|x| str::trim(x).to_string())).collect();
                                prop.clean = Some(Exec::Fn(fn_body.to_string(), p_map));
                                brk = true;
                                break;
                            }
                            if brk{
                                break;
                            }
                        }
                    }
                } else {
                    prop.clean = Some(Exec::Cmd(v_prop[4].to_string()))
                }
            }

            // EVENTS
            if !v_prop[5].is_empty(){
                prop.events = parse_event(v_prop[5]);
            }

            // MONITOR_DIR
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

    fn parse_event(s: &str) -> u16{
        let mut ret = 0;
        let s = s.to_lowercase();
        for cd in s.as_bytes().chunks(2){
            let code = std::str::from_utf8(cd).unwrap();
            //  events -> Cr De Mo Mv Op Cw Ac At Al
            match code {
                "cr" => {ret |= 1 << 0},
                "de" => {ret |= 1 << 1},
                "mo" => {ret |= 1 << 2},
                "mv" => {ret |= 1 << 3},
                "op" => {ret |= 1 << 4},
                "cw" => {ret |= 1 << 5},
                "ac" => {ret |= 1 << 6},
                "at" => {ret |= 1 << 7},
                _ => {ret |= u8::MAX as u16},
            }    
        }
        ret
    }



    #[derive(Debug,PartialEq,PartialOrd)]
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
        
        let mut n_body = "".to_string();

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
            }else{
                n_body.push_str(line);
                n_body.push_str("\n");
            }
        }

        let mut body = n_body;

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

        let lines: Vec<&str> = body.lines().map(str::trim).filter(|x| !x.is_empty()).collect();

        if lines.is_empty() {
            return t_Return::Null;
        }
    
        let mut case_body = vec![];
        let mut get_body = false;
        let mut exit_ladder = false;

        for line in lines{
            if exit_ladder{
                if line.starts_with("FI"){
                    get_body = false;
                    exit_ladder = false;
                    if let Some(ret) = eval_branch(&case_body){
                        return ret; 
                    }
                }else{
                    continue;
                }
            }

            if get_body{
                if line.starts_with("ELIF") || line.starts_with("ELSE") || line.starts_with("FI"){
                    get_body = false;                    
                    exit_ladder = true;
                }else{
                    case_body.push(line);
                }
                continue
            }

            if line.starts_with("IF") || line.starts_with("ELIF"){
                let cond = eval_cond(&line[line.find(" ").unwrap()+1..]);
                if cond{
                    get_body = true;
                    exit_ladder = false;
                    case_body.clear();
                }
            }

            if line.starts_with("RETURN"){
                return eval_half(&line[line.find(" ").unwrap()+1..]);
            }
        }

        t_Return::Null
    }

    fn eval_branch(lines: &Vec<&str>) -> Option<t_Return>{ 
        for line in lines{
            if line.starts_with("RETURN"){
                let ret_str = &line[line.find(" ").unwrap()+1..];
                if ret_str.starts_with("CMD"){
                    let out = Command::new(&line[4..]).output().expect("Stuck and Terminated");
                    return Some(t_Return::Str(if out.status.success(){String::from_utf8(out.stdout).unwrap()} else{String::from_utf8(out.stderr).unwrap()}));
                }
                return Some(eval_half(ret_str));
            } else if line.starts_with("CMD") {
                let _ = Command::new(&line[4..]).output().expect("Stuck and Terminated");
            }
        }
        None
    }


    fn eval_half(half: &str) -> t_Return{
        let half = half.trim();
        let mut c = 0;
        if half.bytes().all(|x| {
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
                return t_Return::Float(half.parse::<f64>().expect("NOT POSSIBLE"));
            }else{
                return t_Return::Int(half.parse::<isize>().expect("NOT POSSIBLE"));
            }
        } else{
            let ret = match half{
                "true" => Some(true),
                "false" => Some(false),
                _ => None
            };
            
            if let Some(z) = ret{
                return t_Return::Bool(z);
            }else{
                if matches!(half,"\"\""|"\'\'"|""){
                    return t_Return::Null
                }else{
                    return t_Return::Str(half.to_string())
                }            
            }
        }
    }

    fn eval_cond(cond: &str) -> bool{
        let cond = cond.trim();

        if let Some((lhs,rhs)) = cond.split_once("=="){
            return eval_half(lhs) == eval_half(rhs);
        }

        if let Some((lhs,rhs)) = cond.split_once("!="){
            return eval_half(lhs) != eval_half(rhs);
        }

        if let Some((lhs,rhs)) = cond.split_once(">"){
            return eval_half(lhs) > eval_half(rhs);
        }

        if let Some((lhs,rhs)) = cond.split_once("<"){
            return eval_half(lhs) < eval_half(rhs);
        }

        if let Some((lhs,rhs)) = cond.split_once("<="){
            return eval_half(lhs) <= eval_half(rhs);
        }

        if let Some((lhs,rhs)) = cond.split_once(">="){
            return eval_half(lhs) >= eval_half(rhs);
        }

        false
    }


}