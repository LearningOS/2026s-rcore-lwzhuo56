# Lab 2：虚拟内存管理

## 实验内容

实现用户地址空间中的 `mmap`、`munmap`、`sbrk` 和跨页 `get_time`。

## 关键设计

- `mmap` 进行页对齐、权限和区域重叠检查。
- 映射区域由 `MapArea` 管理，物理帧生命周期由 `FrameTracker` 自动回收。
- `munmap` 释放目标区域并清理页表与数据帧。
- 用户缓冲区通过页表翻译后分段拷贝，避免直接解引用用户指针。

## 验证

通过 mmap、unmap、sbrk 和 trace 测例。
