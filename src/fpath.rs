pub mod Fpath{
    use std::{collections::HashMap, path::{Component, Components, Path}, sync::{Arc, Mutex}};

    pub struct NTree{
        prefix: String,
        root: Option<Arc<Mutex<NNode>>>
    }

    impl NTree{
        pub fn new(pfx: Option<String>) ->  Self{
            Self { prefix: pfx.unwrap_or(std::env::current_dir().unwrap().to_str().unwrap().to_string()), root: None}

        }

        pub fn add_path(&mut self,pth: &Path) {
            if let Some(cur) = &self.root{
                self._add_path(&mut pth.canonicalize().unwrap().components().peekable(),cur);
            }else{
                let mut iterab = &mut pth.canonicalize().unwrap().components().peekable();
                match iterab.next(){
                    Some(item) => {
                        let comp = match item{
                            Component::RootDir => "/",
                            Component::Normal(x) => x.to_str().unwrap()
                            _ => {panic!("NOT POSSIBLE");}
                        };
                        self.root = Some(Arc::new(Mutex::new(NNode::new(comp))));
                    },
                    None => {},
                }
            }
        }

        pub fn _add_path<'a,T: Iterator<Item = Component<'a>>>(&mut self,st: &mut T,cur: &Arc<Mutex<NNode>>) {
            if let Some(z) = st.next(){
                let comp = match z{
                    Component::RootDir => "/",
                    Component::Normal(x) => x.to_str().unwrap(),
                    _ => {panic!("NOT POSSIBLE")}
                };
                if let Some((nxt,is_dir)) = cur.children.get(comp){

                
                }else{
                    cur.children.insert(comp.to_string(),(Arc::new(Mutex::new(NNode::new())),s));
                }
                    
            }else{

            }
        }

    }

    pub struct NNode{
        pub me: String,
        pub children: HashMap<String,(Arc<Mutex<NNode>>,bool)>
    }

    impl NNode{
        pub fn new(me: &str) -> Self{
            Self{children: HashMap::new(),me:me.to_string()}
        }

    }






}