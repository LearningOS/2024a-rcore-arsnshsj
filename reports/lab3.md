## Lab3 报告
### 功能实现
+ 实现了sys_spawn函数
    + spawn相比fork子进程有了独立于父进程的空间
+ 实现了stride调度
    + 直接记录加暴力扫

### 问题回答
1. 
+ 不是，p2的255加10溢出了
2.
+ pass=BigStride/prio，因为prio>=2，所以pass<=BigStride/2
+ 因为MAX-MIN<=BigStride/2
+ 调度之后的MIN'>=MIN
+ MAX'=MAX(MAX, MIN+BigStride/2)
+ MAX'-MIN'<=MAX(MAX, MIN+BigStride/2)-MIN<=BigStride/2
3.
```rust
// 假设bigstride的最大值为max
impl PartialOrd for Stride {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let mid = (max/2) + 1;
        if self.0 > other.0 {
            if (self.0-other.0) >= mid {
                Ordering::Less
            } else {
                Ordering::Greater
            }
        } else {
            if (other.0-self.0) >= mid {
                Ordering::Greater
            } else {
                Ordering::Less
            }
        }
    }
}
```
### 荣誉准则
1. 在完成本次实验的过程（含此前学习的过程）中，我曾分别与 以下各位 就（与本次实验相关的）以下方面做过交流，还在代码中对应的位置以注释形式记录了具体的交流对象及内容：

    无

2. 此外，我也参考了 以下资料 ，还在代码中对应的位置以注释形式记录了具体的参考来源及内容：

    https://learningos.cn/rCore-Camp-Guide-2024A/  

3. 我独立完成了本次实验除以上方面之外的所有工作，包括代码与文档。 我清楚地知道，从以上方面获得的信息在一定程度上降低了实验难度，可能会影响起评分。

4. 我从未使用过他人的代码，不管是原封不动地复制，还是经过了某些等价转换。 我未曾也不会向他人（含此后各届同学）复制或公开我的实验代码，我有义务妥善保管好它们。 我提交至本实验的评测系统的代码，均无意于破坏或妨碍任何计算机系统的正常运转。 我清楚地知道，以上情况均为本课程纪律所禁止，若违反，对应的实验成绩将按“-100”分计。