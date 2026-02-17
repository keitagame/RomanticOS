use alloc::string::String;

use alloc::collections::BTreeMap;
use alloc::boxed::Box;
use spin::Mutex;
use alloc::vec;
use alloc::vec::Vec;

const MAX_OPEN_FILES: usize = 1024;
const MAX_FILE_SIZE: usize = 1024 * 1024; // 1MB

static FILESYSTEM: Mutex<Option<VirtualFileSystem>> = Mutex::new(None);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileType {
    Regular,
    Directory,
    Device,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FileMode {
    pub read: bool,
    pub write: bool,
    pub execute: bool,
}

impl FileMode {
    pub fn from_bits(mode: u32) -> Self {
        Self {
            read: (mode & 0o400) != 0,
            write: (mode & 0o200) != 0,
            execute: (mode & 0o100) != 0,
        }
    }
}
#[derive(Clone)]
pub struct Inode {
    pub inode_num: usize,
    pub file_type: FileType,
    pub mode: FileMode,
    pub size: usize,
    pub data: Vec<u8>,
    pub children: BTreeMap<String, usize>, // ディレクトリの場合
}

impl Inode {
    fn new_file(inode_num: usize, mode: FileMode) -> Self {
        Self {
            inode_num,
            file_type: FileType::Regular,
            mode,
            size: 0,
            data: Vec::new(),
            children: BTreeMap::new(),
        }
    }

    fn new_dir(inode_num: usize, mode: FileMode) -> Self {
        Self {
            inode_num,
            file_type: FileType::Directory,
            mode,
            size: 0,
            data: Vec::new(),
            children: BTreeMap::new(),
        }
    }
}
#[derive(Clone)]
pub struct OpenFile {
    pub inode: usize,
    pub offset: usize,
    pub flags: i32,
}

pub struct VirtualFileSystem {
    inodes: Vec<Option<Inode>>,
    open_files: Vec<Option<OpenFile>>,
    next_inode: usize,
    root_inode: usize,
}

impl VirtualFileSystem {
    fn new() -> Self {
        let mut vfs = Self {
            inodes: vec![None; 1024],
            open_files: vec![None; MAX_OPEN_FILES],
            next_inode: 1,
            root_inode: 0,
        };

        // ルートディレクトリを作成
        let root = Inode::new_dir(0, FileMode {
            read: true,
            write: true,
            execute: true,
        });
        vfs.inodes[0] = Some(root);

        vfs
    }

    fn allocate_inode(&mut self) -> Option<usize> {
        let inode_num = self.next_inode;
        if inode_num >= self.inodes.len() {
            return None;
        }
        self.next_inode += 1;
        Some(inode_num)
    }

    fn allocate_fd(&mut self) -> Option<usize> {
        for (i, slot) in self.open_files.iter().enumerate() {
            if slot.is_none() {
                return Some(i);
            }
        }
        None
    }

    pub fn create(&mut self, path: &str, mode: FileMode) -> Result<usize, &'static str> {
        let parts: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
        
        if parts.is_empty() {
            return Err("Invalid path");
        }

        let filename = parts[parts.len() - 1];
        let parent_inode = self.traverse_path(&parts[..parts.len() - 1])?;

        // 既に存在するかチェック
        if let Some(parent) = &self.inodes[parent_inode] {
            if parent.children.contains_key(filename) {
                return Err("File already exists");
            }
        }

        // 新しいinodeを割り当て
        let inode_num = self.allocate_inode().ok_or("Out of inodes")?;
        let inode = Inode::new_file(inode_num, mode);
        self.inodes[inode_num] = Some(inode);

        // 親ディレクトリに追加
        if let Some(parent) = &mut self.inodes[parent_inode] {
            parent.children.insert(String::from(filename), inode_num);
        }

        Ok(inode_num)
    }

    pub fn mkdir(&mut self, path: &str, mode: FileMode) -> Result<usize, &'static str> {
        let parts: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
        
        if parts.is_empty() {
            return Err("Invalid path");
        }

        let dirname = parts[parts.len() - 1];
        let parent_inode = self.traverse_path(&parts[..parts.len() - 1])?;

        let inode_num = self.allocate_inode().ok_or("Out of inodes")?;
        let inode = Inode::new_dir(inode_num, mode);
        self.inodes[inode_num] = Some(inode);

        if let Some(parent) = &mut self.inodes[parent_inode] {
            parent.children.insert(String::from(dirname), inode_num);
        }

        Ok(inode_num)
    }

    fn traverse_path(&self, parts: &[&str]) -> Result<usize, &'static str> {
        let mut current = self.root_inode;

        for part in parts {
            if let Some(inode) = &self.inodes[current] {
                if inode.file_type != FileType::Directory {
                    return Err("Not a directory");
                }
                current = *inode.children.get(*part).ok_or("Path not found")?;
            } else {
                return Err("Invalid inode");
            }
        }

        Ok(current)
    }

    pub fn open(&mut self, path: &str, flags: i32) -> Result<i32, &'static str> {
        let parts: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
        let inode_num = self.traverse_path(&parts)?;

        let fd = self.allocate_fd().ok_or("Too many open files")? as i32;
        
        self.open_files[fd as usize] = Some(OpenFile {
            inode: inode_num,
            offset: 0,
            flags,
        });

        Ok(fd)
    }

    pub fn close(&mut self, fd: i32) -> Result<(), &'static str> {
        if fd < 0 || fd as usize >= self.open_files.len() {
            return Err("Invalid file descriptor");
        }

        self.open_files[fd as usize] = None;
        Ok(())
    }

    pub fn read(&mut self, fd: i32, buf: &mut [u8]) -> Result<usize, &'static str> {
        if fd < 0 || fd as usize >= self.open_files.len() {
            return Err("Invalid file descriptor");
        }

        let open_file = self.open_files[fd as usize].as_mut()
            .ok_or("File not open")?;

        let inode = self.inodes[open_file.inode].as_ref()
            .ok_or("Invalid inode")?;

        if !inode.mode.read {
            return Err("Permission denied");
        }

        let start = open_file.offset;
        let end = core::cmp::min(start + buf.len(), inode.data.len());
        let bytes_read = end - start;

        buf[..bytes_read].copy_from_slice(&inode.data[start..end]);
        open_file.offset = end;

        Ok(bytes_read)
    }

    pub fn write(&mut self, fd: i32, buf: &[u8]) -> Result<usize, &'static str> {
        if fd < 0 || fd as usize >= self.open_files.len() {
            return Err("Invalid file descriptor");
        }

        let inode_num = {
            let open_file = self.open_files[fd as usize].as_ref()
                .ok_or("File not open")?;
            open_file.inode
        };

        let inode = self.inodes[inode_num].as_mut()
            .ok_or("Invalid inode")?;

        if !inode.mode.write {
            return Err("Permission denied");
        }

        let open_file = self.open_files[fd as usize].as_mut().unwrap();
        let start = open_file.offset;

        // データを拡張
        if start + buf.len() > inode.data.len() {
            if start + buf.len() > MAX_FILE_SIZE {
                return Err("File too large");
            }
            inode.data.resize(start + buf.len(), 0);
        }

        inode.data[start..start + buf.len()].copy_from_slice(buf);
        inode.size = core::cmp::max(inode.size, start + buf.len());
        open_file.offset = start + buf.len();

        Ok(buf.len())
    }

    pub fn list_dir(&self, path: &str) -> Result<Vec<String>, &'static str> {
        let parts: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
        let inode_num = self.traverse_path(&parts)?;

        let inode = self.inodes[inode_num].as_ref()
            .ok_or("Invalid inode")?;

        if inode.file_type != FileType::Directory {
            return Err("Not a directory");
        }

        Ok(inode.children.keys().cloned().collect())
    }
}

pub fn init() {
    let mut vfs = VirtualFileSystem::new();

    // いくつかのディレクトリを作成
    vfs.mkdir("/dev", FileMode { read: true, write: true, execute: true }).ok();
    vfs.mkdir("/tmp", FileMode { read: true, write: true, execute: true }).ok();
    vfs.mkdir("/home", FileMode { read: true, write: true, execute: true }).ok();

    // テストファイルを作成
    vfs.create("/hello.txt", FileMode { read: true, write: true, execute: false }).ok();

    *FILESYSTEM.lock() = Some(vfs);
}

// グローバルAPI
pub fn open(path: &str, flags: i32, _mode: u32) -> i64 {
    let mut fs = FILESYSTEM.lock();
    if let Some(fs) = fs.as_mut() {
        match fs.open(path, flags) {
            Ok(fd) => fd as i64,
            Err(_) => -1,
        }
    } else {
        -1
    }
}

pub fn close(fd: i32) -> i64 {
    let mut fs = FILESYSTEM.lock();
    if let Some(fs) = fs.as_mut() {
        match fs.close(fd) {
            Ok(_) => 0,
            Err(_) => -1,
        }
    } else {
        -1
    }
}

pub fn read(fd: i32, buf: &mut [u8]) -> i64 {
    let mut fs = FILESYSTEM.lock();
    if let Some(fs) = fs.as_mut() {
        match fs.read(fd, buf) {
            Ok(n) => n as i64,
            Err(_) => -1,
        }
    } else {
        -1
    }
}

pub fn write(fd: i32, buf: &[u8]) -> i64 {
    let mut fs = FILESYSTEM.lock();
    if let Some(fs) = fs.as_mut() {
        match fs.write(fd, buf) {
            Ok(n) => n as i64,
            Err(_) => -1,
        }
    } else {
        -1
    }
}

pub fn create_file(path: &str) -> Result<(), &'static str> {
    let mut fs = FILESYSTEM.lock();
    if let Some(fs) = fs.as_mut() {
        fs.create(path, FileMode { read: true, write: true, execute: false })?;
        Ok(())
    } else {
        Err("Filesystem not initialized")
    }
}

pub fn list_directory(path: &str) -> Result<Vec<String>, &'static str> {
    let fs = FILESYSTEM.lock();
    if let Some(fs) = fs.as_ref() {
        fs.list_dir(path)
    } else {
        Err("Filesystem not initialized")
    }
}
