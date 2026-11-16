# Lab 3：进程管理

本实验实现进程加载、派生、等待和优先级设置。`spawn` 从 ELF 创建独立进程，维护父子关系并加入调度器；`set_priority` 拒绝小于 2 的优先级。

验证：通过 spawn、waitpid、fork、exit 与 set_priority 测例。
