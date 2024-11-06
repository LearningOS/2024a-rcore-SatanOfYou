# ch6 实验实现总结
1：linkat 的实现很简单，只需要判断原文件是否存在。存在拿到它的 inode index，然后在ROOT inode 下多写入一个DIRENTRY，名字是新文件名，inode号就是旧文件的 inode index。

2：unlinkat 的实现比较难，目前的实现是：直接判断文件是否存在，存在，那么就直接在ROOT inode下清除它的 DIRENTRY内容。清除之后从后向前把DIRENTRY覆盖过去。然后设置 disk inode的大小减去DIRENTRY的size。只是找不到文件了现在。

这时候，其实没考虑ROOT inode的datablock的回收。因为如果DIRENTRY在一个datablock的边界位置，那只是清除了它的内容。

在注释内容里还实现了目标文件的file inode的clear函数调用，回收所有的data block以及涉及到的inode中indirect部分的datablock。但是这时候没回收文件申请的inode。

但是一个可能好的实现：回收掉该文件的inode，因为申请新建文件的时候，申请inode会调用它的clear，也就是旧文件的内容，新文件来打扫。不需要在unlink的时候调用clear。

3：stat的实现：给 File这个trait添加了ino，is_inode_is_dir，nlink等方法，获取所需的信息，然后直接拷贝到用户空间。但这种做法不具有扩展性。不太好

# 问答作业
## 在我们的easy-fs中，root inode起着什么作用？如果root inode中的内容损坏了，会发生什么？

ROOT inode 起最重要的作用，所有文件只有一级，都位于ROOT inode下。在Linux文件结构中，它代表 / 所在的inode。

如果 ROOT inode内容损坏，会导致所有的文件不能正确读写，找不到正确的文件内容。文件系统全部崩溃。

