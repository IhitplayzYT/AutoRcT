pub mod Fpath{
use std::{collections::HashMap,path::{Component, Path, PathBuf},sync::{Arc, Mutex}};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeType {
    File,
    Directory,
}

#[derive(Debug)]
pub struct NNode {
    pub name: String,
    pub node_type: NodeType,
    pub children: HashMap<String, Arc<Mutex<NNode>>>,
}

impl NNode {
    pub fn new(name: impl Into<String>, node_type: NodeType) -> Self {
        Self {name: name.into(),node_type,children: HashMap::new()}
    }

    pub fn is_dir(&self) -> bool {
        self.node_type == NodeType::Directory
    }

    pub fn is_file(&self) -> bool {
        self.node_type == NodeType::File
    }
}

#[derive(Debug, Clone)]
pub struct NTree {
    pub prefix: PathBuf,
    pub root: Arc<Mutex<Option<Arc<Mutex<NNode>>>>>,
}

impl NTree {
    pub fn new(pfx: Option<String>) -> Self {
        let prefix = match pfx {
            Some(p) => PathBuf::from(p),
            None => std::env::current_dir().expect("failed to get current directory"),
        };

        Self {prefix,root: Arc::new(Mutex::new(None))}
    }

    fn _component_name(component: Component<'_>) -> Option<String> {
        match component {
            Component::RootDir => Some("/".to_string()),
            Component::Normal(x) => {Some(x.to_string_lossy().into_owned())}
            Component::CurDir => None,
            Component::ParentDir => {Some("..".to_string())}
            Component::Prefix(x) => {Some(x.as_os_str().to_string_lossy().into_owned())}
        }
    }

    fn components(path: &Path) -> Vec<String> {
        path.components().filter_map(Self::_component_name).collect()
    }

    fn node_type(path: &Path) -> NodeType {
        if path.is_dir() {
            NodeType::Directory
        } else {
            NodeType::File
        }
    }

    pub fn add_path(&self, path: impl AsRef<Path>) -> std::io::Result<()> {
        let path = path.as_ref();
        let canonical = path.canonicalize()?;
        let components = Self::components(&canonical);

        if components.is_empty() {
            return Ok(());
        }

        let final_type = Self::node_type(&canonical);
        let mut root_guard = self.root.lock().unwrap();
        if root_guard.is_none() {
            let first = &components[0];
            let root_type = if components.len() == 1 {
                final_type
            } else {
                NodeType::Directory
            };
            *root_guard = Some(Arc::new(Mutex::new(NNode::new(first.clone(), root_type))));
        }
        let root = root_guard.as_ref().unwrap().clone();
        drop(root_guard);
        self.add_components(root,&components[1..],final_type);
        Ok(())
    }

    fn add_components(&self,mut current: Arc<Mutex<NNode>>,components: &[String],final_type: NodeType) {
        for (index, name) in components.iter().enumerate() {
            let last = index == components.len() - 1;
            let mut node = current.lock().unwrap();
            if !node.is_dir() {
                return;
            }
            let child = match node.children.get(name) {
                Some(existing) => existing.clone(),
                None => {
                    let ty = if last {
                        final_type
                    } else {
                        NodeType::Directory
                    };
                    let new_node = Arc::new(Mutex::new(
                        NNode::new(name.clone(), ty)
                    ));
                    node.children.insert(name.clone(), new_node.clone());
                    new_node
                }
            };
            drop(node);
            current = child;
        }
    }

    pub fn find(&self, path: impl AsRef<Path>) -> Option<Arc<Mutex<NNode>>> {
        let components = Self::components(path.as_ref());
        if components.is_empty() {
            return None;
        }
        let root_guard = self.root.lock().unwrap();
        let mut current = root_guard.as_ref()?.clone();
        drop(root_guard);
        for component in components.iter().skip(1) {
            let node = current.lock().unwrap();
            if !node.is_dir() {
                return None;
            }
            let child = node.children.get(component)?.clone();
            drop(node);
            current = child;
        }
        Some(current)
    }

    pub fn exists(&self, path: impl AsRef<Path>) -> bool {
        self.find(path).is_some()
    }

    pub fn is_dir(&self, path: impl AsRef<Path>) -> bool {
        match self.find(path) {
            Some(node) => node.lock().unwrap().is_dir(),
            None => false,
        }
    }

    pub fn is_file(&self, path: impl AsRef<Path>) -> bool {
        match self.find(path) {
            Some(node) => node.lock().unwrap().is_file(),
            None => false,
        }
    }

    pub fn delete_path(&self,path: impl AsRef<Path>) -> bool {
        let components = Self::components(path.as_ref());
        if components.len() <= 1 {
            let mut root = self.root.lock().unwrap();
            if root.is_some() {
                *root = None;
                return true;
            }
            return false;
        }

        let root_guard = self.root.lock().unwrap();
        let mut current = match root_guard.as_ref() {
            Some(root) => root.clone(),
            None => return false,
        };
        drop(root_guard);

        for component in &components[1..components.len() - 1] {
            let node = current.lock().unwrap();
            let child = match node.children.get(component) {
                Some(x) => x.clone(),
                None => return false,
            };
            drop(node);
            current = child;
        }
        let target = &components[components.len() - 1];
        let mut parent = current.lock().unwrap();
        parent.children.remove(target).is_some()
    }

    pub fn update_path(&self,old_path: impl AsRef<Path>,new_path: impl AsRef<Path>) -> bool {
        let old_components = Self::components(old_path.as_ref());
        let new_components = Self::components(new_path.as_ref());
        if old_components.is_empty() || new_components.is_empty() {
            return false;
        }
        if old_components.len() == 1 {
            return false;
        }
        let old_name = old_components.last().unwrap().clone();
        let old_parent_components = &old_components[..old_components.len() - 1];
        let new_parent_components = &new_components[..new_components.len() - 1];
        let new_name = new_components.last().unwrap().clone();
        let old_parent = match self.find_components(old_parent_components) {
            Some(x) => x,
            None => return false,
        };
        let new_parent = match self.find_components(new_parent_components) {
            Some(x) => x,
            None => return false,
        };
        let node = {
            let mut parent = old_parent.lock().unwrap();
            match parent.children.remove(&old_name) {
                Some(node) => node,
                None => return false,
            }
        };
        {
            let mut n = node.lock().unwrap();
            n.name = new_name.clone();
        }

        {
            let mut parent = new_parent.lock().unwrap();
            if !parent.is_dir() {
                return false;
            }
            parent.children.insert(new_name, node);
        }

        true
    }

    fn find_components(&self,components: &[String],) -> Option<Arc<Mutex<NNode>>> {
        if components.is_empty() {
            return None;
        }
        let root_guard = self.root.lock().unwrap();
        let mut current = root_guard.as_ref()?.clone();
        drop(root_guard);
        for component in components.iter().skip(1) {
            let node = current.lock().unwrap();
            let child = node.children.get(component)?.clone();
            drop(node);
            current = child;
        }
        Some(current)
    }

    pub fn append_path(&self,base: impl AsRef<Path>,suffix: impl AsRef<Path>,) -> PathBuf {
        base.as_ref().join(suffix.as_ref())
    }

    // Removes n components from the end.
    pub fn trim_last_n(&self,path: impl AsRef<Path>,count: usize) -> PathBuf {
        let components: Vec<_> =path.as_ref().components().collect();
        if count >= components.len() {
            return PathBuf::new();
        }
        let mut result = PathBuf::new();
        for component in components.iter().take(components.len() - count){
            result.push(component.as_os_str());
        }
        result
    }

    pub fn children(&self,path: impl AsRef<Path>) -> Vec<String> {
        let Some(node) = self.find(path) else {
            return Vec::new();
        };
        let node = node.lock().unwrap();
        node.children.keys().cloned().collect()
    }

    pub fn clear(&self) {
        let mut root = self.root.lock().unwrap();
        *root = None;
    }
}


}