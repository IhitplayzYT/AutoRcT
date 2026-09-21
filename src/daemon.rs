pub mod daemon{
    use std::collections::{HashMap, HashSet};
    use std::panic::panic_any;
use std::path::PathBuf;
    use std::path::Path; 
    use std::thread;
    use inotify::{EventMask, Inotify, WatchDescriptor, WatchMask};

use crate::parse::Parse::{Config, Properties};

    pub struct InotifyMonitor {
        inotify: Inotify,
        watches: HashMap<WatchDescriptor, PathBuf>,
        ignored_files: HashSet<PathBuf>,
        ignored_dirs: HashSet<PathBuf>,
        conf:HashMap<(String,String), Properties>,
        ftree: NTree<String>,
    }

    impl InotifyMonitor {

        pub fn new(conf : Config) -> std::io::Result<Self> {
            let mut ret = Self{inotify: Inotify::init()?,watches: HashMap::new(),ignored_files: HashSet::new(),ignored_dirs: HashSet::new(),conf:conf.exec_props};
            let ignores = conf.ignore_files;
            ignores.iter().for_each(|x| {
                let pth = PathBuf::from(x);
                if pth.exists(){
                    if pth.is_dir(){
                        ret.ignored_dirs.insert(pth);
                    }else{
                        ret.ignored_files.insert(pth);
                    }
                }
            });

            Ok(ret)
        }

        pub fn ignore_file(&mut self, path: &Path) {
            self.ignored_files.insert(path.to_path_buf());
        }

        pub fn ignore_dir(&mut self, path: &Path) {
            self.ignored_dirs.insert(path.to_path_buf());
        }

        fn is_ignored_dir(&self, path: &Path) -> bool {
            self.ignored_dirs.iter().any(|ignored| path == ignored || path.starts_with(ignored))
        }

        fn is_ignored_file(&self, path: &Path) -> bool {
            self.ignored_files.contains(path)
        }

        pub fn watch_dir(&mut self,dir: &Path,mask: WatchMask) -> std::io::Result<()> {
            if self.is_ignored_dir(dir) {
                return Ok(());
            }
            let wd = self.inotify.watches().add(dir, mask)?;
            self.watches.insert(wd,dir.to_path_buf(),);
            Ok(())
        }

        pub fn run(mut self) -> thread::JoinHandle<()> {
            thread::spawn(move || {
                let mut buffer = [0u8; 16 * 1024];
                loop {
                    let events = match self.inotify.read_events_blocking(&mut buffer){
                        Ok(events) => events,
                        Err(e) => {
                            eprintln!("inotify read error: {e}");
                            break;
                        }
                    };

                    for event in events {
                        let base = match self.watches.get(&event.wd) {
                            Some(base) => base,
                            None => continue,
                        };

                        let path = match event.name {
                            Some(name) => base.join(name),
                            None => base.clone(),
                        };
                        println!("{base:?} {path:?}");

                        if self.is_ignored_file(&path) ||self.is_ignored_dir(&path) {
                            continue;
                        }
                        self.handle_event(&path,event.mask);
                    }
                }
            })
        }

        fn handle_event(&mut self,path: &Path,mask: EventMask) {
            if mask.contains(EventMask::CREATE) {
                println!("CREATE  {}", path.display());
                
                if mask.contains(EventMask::CREATE) && mask.contains(EventMask::ISDIR) && !self.is_ignored_dir(path){
                    let mask = WatchMask::CREATE | WatchMask::DELETE | WatchMask::MODIFY | WatchMask::MOVED_FROM | WatchMask::MOVED_TO;
                    if let Err(e) = self.watch_dir(path, mask) {
                        eprintln!("failed to watch {}: {}",path.display(),e);
                    }
                }
            }

            if mask.contains(EventMask::MODIFY) {
            }

            if mask.contains(EventMask::DELETE) {
            }

            if mask.contains(EventMask::MOVED_FROM) {
            }

            if mask.contains(EventMask::MOVED_TO) {
            }
        }

    }

    #[derive(Debug, Clone, Copy)]
    pub enum MonitorEvent {
        Create,
        Delete,
        Modify,
        Move,
        Open,
        CloseWrite,
        Access,
        Attribute,
        All,
    }


    impl MonitorEvent {
        fn mask(self) -> WatchMask {
            match self {
                Self::Create => WatchMask::CREATE,
                Self::Delete => WatchMask::DELETE,
                Self::Modify => WatchMask::MODIFY,
                Self::Move => { WatchMask::MOVED_FROM | WatchMask::MOVED_TO}
                Self::Open => WatchMask::OPEN,
                Self::CloseWrite => WatchMask::CLOSE_WRITE,
                Self::Access => WatchMask::ACCESS,
                Self::Attribute => WatchMask::ATTRIB,
                Self::All => WatchMask::ALL_EVENTS,
            }
        }
    }

}