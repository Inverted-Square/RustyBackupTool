use copy_dir::copy_dir;
use std::collections::{HashMap, HashSet};
use std::ffi::OsString;
use std::fs::{DirEntry, File, ReadDir};
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::time::UNIX_EPOCH;
use std::{fs, io};

pub struct BackupTool {
    ignored_paths: HashSet<PathBuf>,
    display_only: bool,
}

type RemoveFunc = fn(PathBuf) -> io::Result<()>;

impl BackupTool {
    pub fn new(display_only: bool) -> io::Result<Self> {
        let file = File::open(".backup_ignore")?;
        let reader = BufReader::new(file);
        let ignored_paths = reader
            .lines()
            .map_while(Result::ok)
            .map(PathBuf::from)
            .collect();

        Ok(Self {
            ignored_paths,
            display_only,
        })
    }

    pub fn backup(&self, src: &PathBuf, dest: &PathBuf) -> io::Result<()> {
        if !self.ignored_paths.contains(src) {
            self.backup_aux(src, dest)?;
        }

        Ok(())
    }

    fn backup_aux(&self, src: &PathBuf, dest: &PathBuf) -> io::Result<()> {
        let src_metadata = fs::metadata(src)?;
        let dest_metadata = fs::metadata(dest)?;
        match (src_metadata.is_dir(), dest_metadata.is_dir()) {
            (false, false) => self.replace_file(src, dest),
            (false, true) => self.replace_entry(fs::remove_dir_all, src, dest),
            (true, false) => self.replace_entry(fs::remove_file, src, dest),
            (true, true) => self.compare_dirs(fs::read_dir(src)?, dest),
        }
    }

    fn replace_file(&self, src: &PathBuf, dest: &PathBuf) -> io::Result<()> {
        let src_modified = Self::modified_as_secs(src)?;
        let dest_modified = Self::modified_as_secs(dest)?;
        if src_modified <= dest_modified {
            return Ok(());
        }

        println!("Replace '{}' with '{}'", dest.display(), src.display());
        if self.display_only {
            return Ok(());
        }

        fs::copy(src, dest).map(|_| ())
    }

    fn modified_as_secs(path: &PathBuf) -> io::Result<u64> {
        let val = fs::metadata(path)?
            .modified()?
            .duration_since(UNIX_EPOCH)
            .map_err(io::Error::other)?
            .as_secs();
        Ok(val)
    }

    fn replace_entry(&self, f: RemoveFunc, src: &PathBuf, dest: &PathBuf) -> io::Result<()> {
        f(dest.clone())?;
        self.add_entry(src, dest)
    }

    fn add_entry(&self, src: &PathBuf, dest: &PathBuf) -> io::Result<()> {
        println!("Add '{}' to '{}'", src.display(), dest.display());
        if self.display_only || !fs::exists(src)? {
            return Ok(());
        }

        let entry_is_dir = fs::metadata(src)?.is_dir();
        let mut dest = PathBuf::from(dest);
        let src_name = src.file_name().ok_or(io::ErrorKind::InvalidInput)?;
        dest.push(src_name);
        if entry_is_dir {
            copy_dir(src, dest).map(|_| ())
        } else {
            fs::copy(src, dest).map(|_| ())
        }
    }

    fn compare_dirs(&self, src_dir: ReadDir, dest: &PathBuf) -> io::Result<()> {
        let mut dest_entries = fs::read_dir(dest)?
            .flatten()
            .map(|entry| (entry.file_name(), entry))
            .collect::<HashMap<_, _>>();

        for src_entry in src_dir.flatten() {
            if self.ignored_paths.contains(&src_entry.path()) {
                continue;
            }

            match dest_entries.remove(&src_entry.file_name()) {
                Some(dest_entry) => self.backup_aux(&src_entry.path(), &dest_entry.path())?,
                None => self.add_entry(&src_entry.path(), dest)?,
            }
        }

        self.remove_remaining(dest_entries)
    }

    fn remove_remaining(&self, dest_entries: HashMap<OsString, DirEntry>) -> io::Result<()> {
        for (_, entry) in dest_entries {
            let path = entry.path();
            println!("Remove '{}'", path.display());
            if self.display_only {
                continue;
            }

            let entry_is_dir = entry.metadata()?.is_dir();
            if entry_is_dir {
                fs::remove_dir_all(path)?;
            } else {
                fs::remove_file(path)?;
            }
        }

        Ok(())
    }
}
