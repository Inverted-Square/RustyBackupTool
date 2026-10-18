# Simple Backup Tool

Author: Ashley Leonard

---

## Description

This is a small personal tool to back up files from one folder to another. It compares differences in the contents of the source and destination to decide whether to add, remove, replace or leave them. This currently doesn't check into folders with different names to see if they have identical files so if a large folder gets renamed there will be an expensive remove + add operation.
