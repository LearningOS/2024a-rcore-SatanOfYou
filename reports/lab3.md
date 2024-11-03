# ch5 实验实现总结
1：sys_spawn 根据 sys_exec 和 sys_fork 的具体实现参考结合即可。

2：sys_set_priority 设置优先级。
这里有一个很大的问题：
进程的 stride 初始值为 0，类型为 usize， priority 设置初始值为 16。每次更新内容为 usize::MAX / task.priority。
一般情况下，priority 设置在 15 左右。测试中有设置 priority 很大的情况，只是个例，很快调度执行完毕。

问题：在常规任务调度中，任务调度几次之后，就开始重复调度任务 0：initproc。
主要原因在于：task 0 最先调度，其 stride 会先增加到临界值。而大家的priority 默认值都是 16。导致每次增加的 stride 都很大且每一步都一样。
    那么，当其余任务调度之后，导致其 stride 大于 task 0 的 stride。当大家stride相等时，调度最后一个任务。
    任务调度到一段时间之后，开始溢出，则必定会有两个任务保持不动，另外一个反复调度。

解决办法：每次更新stride进行取余操作。具体可以看 os-output.txt

# 问答题

## 1、实际情况是轮到 p1 执行吗？为什么？

不是，由于无符号溢出，导致其能继续执行。

