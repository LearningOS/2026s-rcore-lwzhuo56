# Lab 4：文件系统接口

本实验扩展 `File` 抽象并实现 `fstat`、`linkat`、`unlinkat`。硬链接复用同一 inode，删除目录项后由 inode 引用计数决定资源回收时机。

验证：文件读写、fstat、硬链接、删除和大量文件描述符测例。
