use crate::backup_tool::BackupTool;
use std::io;
use std::path::PathBuf;

mod backup_tool;

fn main() -> io::Result<()> {
    let backup_tool = BackupTool::new(false)?;
    let src = PathBuf::from("/home/invy");
    let dest = PathBuf::from("/run/media/invy/TOSHIBA EXT/home/invy");
    backup_tool.backup(&src, &dest)
}
