# Lab 2：虚拟内存与 mmap

本实验实现用户区域的 `mmap` 与 `munmap`，维护映射区域、页表和物理帧的一致性。`get_time` 通过翻译后的用户缓冲区跨越页边界写入时间结构。

验证：mmap、unmap、sbrk 与 trace 测例。
