mod init;

use std::env;
use std::ffi::{OsStr, OsString};
use std::iter::Skip;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};
use std::vec::IntoIter;

use async_trait::async_trait;
use bytes::Bytes;
use clap::Parser;
use color_eyre::eyre::Context;
use fuse3::raw::prelude::*;
use fuse3::{MountOptions, Result};
use futures_util::stream;
use futures_util::stream::Iter;
use tokio::signal::unix::SignalKind;
use tracing::{info, instrument};

const CONTENT: &str = "hello world\n";

const PARENT_INODE: u64 = 1;
const FILE_INODE: u64 = 2;
const FILE_NAME: &str = "hello-world.txt";
const PARENT_MODE: u16 = 0o755;
const FILE_MODE: u16 = 0o644;
const TTL: Duration = Duration::from_secs(1);
const STATFS: ReplyStatFs = ReplyStatFs {
    blocks: 1,
    bfree: 0,
    bavail: 0,
    files: 1,
    ffree: 0,
    bsize: 4096,
    namelen: u32::MAX,
    frsize: 0,
};

#[derive(Debug)]
struct FS;

#[async_trait]
impl Filesystem for FS {
    type DirEntryStream = Iter<Skip<IntoIter<Result<DirectoryEntry>>>>;
    type DirEntryPlusStream = Iter<Skip<IntoIter<Result<DirectoryEntryPlus>>>>;

    #[instrument(ret, level = "trace")]
    async fn init(&self, _req: Request) -> Result<()> {
        Ok(())
    }

    #[instrument(ret, level = "trace")]
    async fn destroy(&self, _req: Request) {}

    #[instrument(ret, level = "trace")]
    async fn lookup(&self, _req: Request, parent: u64, name: &OsStr) -> Result<ReplyEntry> {
        if parent != PARENT_INODE {
            return Err(libc::ENOENT.into());
        }

        if name != OsStr::new(FILE_NAME) {
            return Err(libc::ENOENT.into());
        }

        Ok(ReplyEntry {
            ttl: TTL,
            attr: FileAttr {
                ino: FILE_INODE,
                size: CONTENT.len() as u64,
                blocks: 0,
                atime: SystemTime::now().into(),
                mtime: SystemTime::now().into(),
                ctime: SystemTime::now().into(),
                kind: FileType::RegularFile,
                perm: FILE_MODE,
                nlink: 0,
                uid: 0,
                gid: 0,
                rdev: 0,
                blksize: 0,
            },
            generation: 0,
        })
    }

    #[instrument(ret, level = "trace")]
    async fn getattr(
        &self,
        _req: Request,
        inode: u64,
        _fh: Option<u64>,
        _flags: u32,
    ) -> Result<ReplyAttr> {
        if inode == PARENT_INODE {
            Ok(ReplyAttr {
                ttl: TTL,
                attr: FileAttr {
                    ino: PARENT_INODE,
                    size: 0,
                    blocks: 0,
                    atime: SystemTime::now().into(),
                    mtime: SystemTime::now().into(),
                    ctime: SystemTime::now().into(),
                    kind: FileType::Directory,
                    perm: PARENT_MODE,
                    nlink: 0,
                    uid: 0,
                    gid: 0,
                    rdev: 0,
                    blksize: 0,
                },
            })
        } else if inode == FILE_INODE {
            Ok(ReplyAttr {
                ttl: TTL,
                attr: FileAttr {
                    ino: FILE_INODE,
                    size: CONTENT.len() as _,
                    blocks: 0,
                    atime: SystemTime::now().into(),
                    mtime: SystemTime::now().into(),
                    ctime: SystemTime::now().into(),
                    kind: FileType::RegularFile,
                    perm: FILE_MODE,
                    nlink: 0,
                    uid: 0,
                    gid: 0,
                    rdev: 0,
                    blksize: 0,
                },
            })
        } else {
            Err(libc::ENOENT.into())
        }
    }

    #[instrument(ret, level = "trace")]
    async fn open(&self, _req: Request, inode: u64, flags: u32) -> Result<ReplyOpen> {
        if inode != PARENT_INODE && inode != FILE_INODE {
            return Err(libc::ENOENT.into());
        }

        Ok(ReplyOpen { fh: 0, flags })
    }

    #[instrument(level = "trace")]
    async fn read(
        &self,
        _req: Request,
        inode: u64,
        _fh: u64,
        offset: u64,
        size: u32,
    ) -> Result<ReplyData> {
        if inode != FILE_INODE {
            return Err(libc::ENOENT.into());
        }

        if offset as usize >= CONTENT.len() {
            Ok(ReplyData { data: Bytes::new() })
        } else {
            let mut data = &CONTENT.as_bytes()[offset as usize..];

            if data.len() > size as usize {
                data = &data[..size as usize];
            }

            Ok(ReplyData {
                data: Bytes::copy_from_slice(data),
            })
        }
    }

    #[instrument(level = "trace")]
    async fn readdir(
        &self,
        _req: Request,
        inode: u64,
        _fh: u64,
        offset: i64,
    ) -> Result<ReplyDirectory<Self::DirEntryStream>> {
        if inode == FILE_INODE {
            return Err(libc::ENOTDIR.into());
        }

        if inode != PARENT_INODE {
            return Err(libc::ENOENT.into());
        }

        let entries = vec![
            Ok(DirectoryEntry {
                inode: PARENT_INODE,
                kind: FileType::Directory,
                name: OsString::from("."),
                offset: 1,
            }),
            Ok(DirectoryEntry {
                inode: PARENT_INODE,
                kind: FileType::Directory,
                name: OsString::from(".."),
                offset: 2,
            }),
            Ok(DirectoryEntry {
                inode: FILE_INODE,
                kind: FileType::RegularFile,
                name: OsString::from(FILE_NAME),
                offset: 3,
            }),
        ];

        Ok(ReplyDirectory {
            entries: stream::iter(entries.into_iter().skip(offset as usize)),
        })
    }

    #[instrument(ret, level = "trace")]
    async fn access(&self, _req: Request, inode: u64, _mask: u32) -> Result<()> {
        if inode != PARENT_INODE && inode != FILE_INODE {
            return Err(libc::ENOENT.into());
        }

        Ok(())
    }

    #[instrument(level = "trace")]
    async fn readdirplus(
        &self,
        _req: Request,
        parent: u64,
        _fh: u64,
        offset: u64,
        _lock_owner: u64,
    ) -> Result<ReplyDirectoryPlus<Self::DirEntryPlusStream>> {
        if parent == FILE_INODE {
            return Err(libc::ENOTDIR.into());
        }

        if parent != PARENT_INODE {
            return Err(libc::ENOENT.into());
        }

        let entries = vec![
            Ok(DirectoryEntryPlus {
                inode: PARENT_INODE,
                generation: 0,
                kind: FileType::Directory,
                name: OsString::from("."),
                offset: 1,
                attr: FileAttr {
                    ino: PARENT_INODE,
                    size: 0,
                    blocks: 0,
                    atime: SystemTime::now().into(),
                    mtime: SystemTime::now().into(),
                    ctime: SystemTime::now().into(),
                    kind: FileType::Directory,
                    perm: PARENT_MODE,
                    nlink: 0,
                    uid: 0,
                    gid: 0,
                    rdev: 0,
                    blksize: 0,
                },
                entry_ttl: TTL,
                attr_ttl: TTL,
            }),
            Ok(DirectoryEntryPlus {
                inode: PARENT_INODE,
                generation: 0,
                kind: FileType::Directory,
                name: OsString::from(".."),
                offset: 2,
                attr: FileAttr {
                    ino: PARENT_INODE,
                    size: 0,
                    blocks: 0,
                    atime: SystemTime::now().into(),
                    mtime: SystemTime::now().into(),
                    ctime: SystemTime::now().into(),
                    kind: FileType::Directory,
                    perm: PARENT_MODE,
                    nlink: 0,
                    uid: 0,
                    gid: 0,
                    rdev: 0,
                    blksize: 0,
                },
                entry_ttl: TTL,
                attr_ttl: TTL,
            }),
            Ok(DirectoryEntryPlus {
                inode: FILE_INODE,
                generation: 0,
                kind: FileType::Directory,
                name: OsString::from(FILE_NAME),
                offset: 3,
                attr: FileAttr {
                    ino: FILE_INODE,
                    size: CONTENT.len() as _,
                    blocks: 0,
                    atime: SystemTime::now().into(),
                    mtime: SystemTime::now().into(),
                    ctime: SystemTime::now().into(),
                    kind: FileType::RegularFile,
                    perm: FILE_MODE,
                    nlink: 0,
                    uid: 0,
                    gid: 0,
                    rdev: 0,
                    blksize: 0,
                },
                entry_ttl: TTL,
                attr_ttl: TTL,
            }),
        ];

        Ok(ReplyDirectoryPlus {
            entries: stream::iter(entries.into_iter().skip(offset as usize)),
        })
    }

    #[instrument(ret, skip(self))]
    async fn statfs(&self, _req: Request, _inode: u64) -> Result<ReplyStatFs> {
        Ok(STATFS)
    }
}

#[derive(Debug, Parser)]
struct Args {
    /// Root mount point
    mount_path: PathBuf,
}

#[tokio::main(flavor = "multi_thread")]
async fn main() -> color_eyre::Result<()> {
    init::init()?;

    let args = Args::parse();
    let mount_path = &args.mount_path;

    let uid = nix::unistd::Uid::current().as_raw();
    let gid = nix::unistd::Gid::current().as_raw();

    let mut mount_options = MountOptions::default();
    mount_options.uid(uid).gid(gid).read_only(true);

    let filesystem = FS {};

    let handle = Session::new(mount_options)
        .mount_with_unprivileged(filesystem, mount_path)
        .await?;

    let task = tokio::spawn(handle);
    tokio::pin!(task);

    let interrupt = tokio::signal::unix::signal(SignalKind::interrupt())?;
    tokio::pin!(interrupt);

    loop {
        tokio::select! {
            res = &mut task => res??,
            _ = interrupt.recv() => {
                info!("Received interrupt signal!");
                break;
            }
        }
    }

    tokio::process::Command::new("fusermount3")
        .arg("-u")
        .arg(mount_path)
        .spawn()
        .wrap_err("Unmounting")?
        .wait()
        .await?;

    Ok(())
}
