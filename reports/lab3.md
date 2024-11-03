# ch5 实验实现总结
1：sys_spawn 根据 sys_exec 和 sys_fork 的具体实现参考结合即可。

2：sys_set_priority 设置优先级。

# 问答题

## 1、实际情况是轮到 p1 执行吗？为什么？

不是，由于无符号溢出，导致其能继续执行。

