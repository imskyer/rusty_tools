use std::fs;

pub struct DevFS {
    entries: Vec<DevFSEntry>,
}

impl DevFS {
    pub fn init(root: &std::path::Path) -> Result<Self, ()> {
        if !root.is_dir() {
            return Err(());
        }
        let current_time = std::time::SystemTime::now();
        let mut entries: Vec<DevFSEntry> = Vec::new();
        DevFS::visit_dir(&mut entries, root, &current_time);
        Ok(DevFS { entries: entries })
    }

    /// Return an iterator over the updated entries.
    pub fn updated_entries(&mut self) -> impl Iterator<Item = &std::path::Path> {
        self.entries.iter_mut().filter_map(|x| {
            if x.is_dirty() {
                Some(x.path.as_path())
            } else {
                None
            }
        })
    }

    fn visit_dir(
        entries: &mut Vec<DevFSEntry>,
        current: &std::path::Path,
        current_time: &std::time::SystemTime,
    ) {
        for entry in fs::read_dir(current).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                DevFS::visit_dir(entries, path.as_path(), current_time);
                continue;
            }
            entries.push(DevFSEntry {
                path: path,
                modified: *current_time,
            });
        }
    }
}

#[derive(Debug)]
pub struct DevFSEntry {
    path: std::path::PathBuf,
    modified: std::time::SystemTime,
}

impl DevFSEntry {
    pub fn is_dirty(&mut self) -> bool {
        let metadata = fs::metadata(&self.path).unwrap();
        let modified = metadata.modified().unwrap();
        if modified > self.modified {
            self.modified = modified;
            true
        } else {
            false
        }
    }
}
