# ch3 实验报告

## 实现总结
1：修改 Task 属性，添加 syscall_count 成员用于系统调用计数，添加 start_time 成员用于记录首次调用时间。

2：修改 Task 成员函数，新建时初始化 syscall_count 为 5 个 0 。在初始设置时仅设置syscall_count 数组长度为系统调用个数。初始化 start_time 为 0 。

3：修改 first_task_run 成员函数，第一个任务调度时设置其 start_time。

4：修改 run_next_task 成员函数，如果下一个任务的 start_time 为 0，则初始化其 start_time 属性。

5：添加 add_syscall_time 成员函数，用于给特定系统调用增加调用次数计数。

6：添加 dump_task_info 成员函数，用于根据当前 task 信息 dump 出 TaskInfo 类型的信息。

7：修改 syscall 函数，在分发特定系统调用前调用 add_syscall_time 添加计数。

8：完善 sys_task_info 函数，调用 dump_task_info 从当前 task 中 dump 出信息。


## 简答作业

### 1、正确进入 U 态后，程序的特征还应有：使用 S 态特权指令，访问 S 态寄存器后会报错。 请同学们可以自行测试这些内容（运行 三个 bad 测例 (ch2b_bad_*.rs) ）， 描述程序出错行为，同时注意注明你使用的 sbi 及其版本。
SBI 版本：RustSBI version 0.3.0-alpha.2

ch2b_bad_address.rs: 写地址 0 ，导致 pagefault。

ch2b_bad_instructions.rs: 在 U 特权级使用 sret 指令，非法。

ch2b_bad_registers.rs: 在 U 特权级读 sstatus 寄存器，非法。

### 2、深入理解 trap.S 中两个函数 __alltraps 和 __restore 的作用，并回答如下问题:
（1）刚进入 __restore 时，a0 代表了什么值。请指出 __restore 的两种使用情景。

刚进入 __restore 时，a0代表系统调用的返回值。

__restore 两种场景：要么从系统调用或者时钟中断中返回，要么任务第一次调度，返回用户态执行。

（2）这几行汇编代码特殊处理了哪些寄存器？这些寄存器的的值对于进入用户态有何意义？请分别解释。

sstatus 以及 sepc 以及 sp寄存器。

sret 执行时会从 sstatus 中获取上一个状态，用于控制系统进入 U 态。

sret 执行，会把 sepc 的值放入 PC，用于控制执行位置。

sp 寄存器用于设置用户栈。

（3）为何跳过了 x2 和 x4？

sp 上文已经设置过了，tp 寄存器不设置。

（4）该指令之后，sp 和 sscratch 中的值分别有什么意义？

sp 指向用户栈，sscratch 指向内核栈。

（5）中发生状态切换在哪一条指令？为何该指令执行之后会进入用户态？

sret 指令。sret 指令会根据 sstatus 的值确定恢复到哪一个状态。而我们为用户程序构造的 sstatus 设置了 SPP 为是 U 态。

（6）L13：该指令之后，sp 和 sscratch 中的值分别有什么意义？

sp 指向内核栈，sscratch 指向用户栈。

（7）从 U 态进入 S 态是哪一条指令发生的？
 
ecall 指令执行之后。