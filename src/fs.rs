use crate::db::{Db, Entry};
use crate::Ino;
use anyhow::{Context, Result};
use fuse::{
    FileAttr, FileType, Filesystem, ReplyAttr, ReplyData, ReplyDirectory, ReplyEntry, Request,
};
use libc::{EIO, ENOENT};
use std::ffi::OsStr;
use std::path::Path;
use std::process::Command;
use std::time::{Duration, UNIX_EPOCH};
use std::{env, fs};

const TTL: Duration = Duration::from_secs(1); // 1 second

const HELLO_DIR_ATTR: FileAttr = FileAttr {
    ino: 1,
    size: 0,
    blocks: 0,
    atime: UNIX_EPOCH, // 1970-01-01 00:00:00
    mtime: UNIX_EPOCH,
    ctime: UNIX_EPOCH,
    crtime: UNIX_EPOCH,
    kind: FileType::Directory,
    perm: 0o755,
    nlink: 2,
    uid: 501,
    gid: 20,
    rdev: 0,
    flags: 0,
};

const HELLO_TXT_CONTENT: &str = "Hello World!\n";

const HELLO_TXT_ATTR: FileAttr = FileAttr {
    ino: 2,
    size: 13,
    blocks: 1,
    atime: UNIX_EPOCH, // 1970-01-01 00:00:00
    mtime: UNIX_EPOCH,
    ctime: UNIX_EPOCH,
    crtime: UNIX_EPOCH,
    kind: FileType::RegularFile,
    perm: 0o644,
    nlink: 1,
    uid: 501,
    gid: 20,
    rdev: 0,
    flags: 0,
};

enum DirectoryItem {
    Entry(Result<(Ino, Entry)>),
    Directory(String),
}

pub struct PlevyFS {
    db: Db,
}

impl PlevyFS {
    pub fn new(db: Db) -> Self {
        Self { db }
    }
}

impl Filesystem for PlevyFS {
    fn lookup(&mut self, _req: &Request, parent: u64, name: &OsStr, reply: ReplyEntry) {
        if parent == 1 {
            let found = self
                .db
                .list()
                .unwrap() // <- todo bad
                .into_iter()
                .filter_map(Result::ok)
                .find(|(_, entry)| entry.name.as_str() == name);

            let (ino, _) = match found {
                Some(entry) => entry,
                None => {
                    reply.error(ENOENT);
                    return;
                }
            };
            let mut attr = HELLO_DIR_ATTR.clone();
            attr.ino = ino;
            reply.entry(&TTL, &attr, 0);
        } else {
            reply.error(ENOENT);
        }
    }

    fn getattr(&mut self, _req: &Request, ino: u64, reply: ReplyAttr) {
        if ino == 1 {
            // root directory
            reply.attr(&TTL, &HELLO_DIR_ATTR);
            return;
        }

        let found = match self.db.has(ino) {
            Ok(entry) => entry,
            Err(err) => {
                warn!("getattr failed: {:?}, replying with IO error...", err);
                reply.error(EIO);
                return;
            }
        };

        if found {
            let mut attr = HELLO_DIR_ATTR.clone();
            attr.ino = ino;
            reply.attr(&TTL, &attr);
        } else {
            reply.error(ENOENT);
        }
    }

    fn read(
        &mut self,
        _req: &Request,
        ino: u64,
        _fh: u64,
        offset: i64,
        _size: u32,
        reply: ReplyData,
    ) {
        if ino == 2 {
            reply.data(&HELLO_TXT_CONTENT.as_bytes()[offset as usize..]);
        } else {
            reply.error(ENOENT);
        }
    }

    fn readdir(
        &mut self,
        _req: &Request,
        ino: u64,
        _fh: u64,
        offset: i64,
        mut reply: ReplyDirectory,
    ) {
        if ino == 1 {
            let mut entries = vec![
                DirectoryItem::Directory(".".to_string()),
                DirectoryItem::Directory("..".to_string()),
            ];
            let listing = match self.db.list() {
                Ok(a) => a,
                Err(err) => {
                    warn!("listing failed: {:?}, replying with IO error...", err);
                    reply.error(EIO);
                    return;
                }
            };
            let mut items: Vec<DirectoryItem> = listing
                .into_iter()
                .map(|entry| DirectoryItem::Entry(entry))
                .collect();
            entries.append(&mut items);

            for (i, item) in entries.iter().enumerate().skip((offset) as usize) {
                match item {
                    DirectoryItem::Directory(name) => {
                        reply.add(1, (i + 1) as i64, FileType::Directory, name);
                    }
                    DirectoryItem::Entry(entry) => {
                        let (ino, entry) = match entry {
                            Ok(e) => e,
                            Err(e) => {
                                reply.error(ENOENT);
                                return;
                            }
                        };
                        debug!("reply -> {}", entry.name);
                        reply.add(
                            ino.clone(),
                            (i + 1) as i64,
                            FileType::Directory,
                            entry.name.as_str(),
                        );
                    }
                }
            }
            reply.ok();
        } else {
            // TODO: directory
            reply.ok();
        }
    }
}

pub fn mount(mount_point: String, db: Db) -> Result<()> {
    let mount_point = mount_point.as_str();
    let options = ["-o", "ro", "-o", "fsname=plevy"]
        .iter()
        .map(|o| o.as_ref())
        .collect::<Vec<&OsStr>>();

    if !Path::new(mount_point).exists() {
        fs::create_dir(mount_point).with_context(|| {
            format!(
                "Attempted to create a directory for our mount point at `{}`, but failed.",
                mount_point
            )
        })?;
    } else {
        // if we get ctrl+c'd, a lingering dead mount will continue to exist which we can't mount to, nor delete.
        // the only solution is unmounting, so attempt that now.

        if Command::new("umount")
            .args(&[mount_point])
            .status()?
            .success()
        {
            warn!(
                "Attempted unmount of {}, but failed. Trying to carry on and still mounting to this path now...",
                mount_point
            )
        } else {
            debug!("Unmounted {}", mount_point);
        }
    }

    fuse::mount(PlevyFS::new(db), mount_point, &options)
        .with_context(|| format!("Can't mount media library file system to `{}`", mount_point))
}
