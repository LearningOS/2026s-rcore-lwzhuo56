# Lab 2：地址空间映射

本实验实现 `mmap`、`munmap` 与跨页用户数据拷贝。映射前检查页对齐、权限和重叠；解除映射时同步更新页表、物理帧和虚存区域。`get_time` 使用翻译后的用户缓冲区分段写入 `TimeVal`。

验证：通过 mmap、unmap、sbrk 和 trace 测例。
