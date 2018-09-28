# VM

## 基本执行过程


## 虚拟机架构

体系结构是基于寄存器的虚拟机，参考了MIPS的风格，也参考了Lua，Python的字节码。

### 寄存器

- 32个32位通用寄存器
- 1个pc
- 1个指令段
- 1个余数专用寄存器
- 1个比较寄存器
- 1个堆

### 指令集

实现如下，计划加入位运算，浮点数支持，函数调用。


```rust
pub enum Opcode {
    /// Load data to register
    LOAD,

    /// Add src1 src2 dst
    ADD,
    /// Sub src1 src2 dst
    SUB,
    /// Mul src1 src2 dst
    MUL,
    /// Div src1 src2 dst
    DIV,
    /// Absolute Jumps by using register
    JMP,
    /// Relative Jumps for jump forwards
    JMPF,
    /// Relative Jumps for jump backwards
    JMPB,

    /// Halt the vm
    HLT,

    /// If src1 == src2 then set true
    EQ,
    /// If src1 != src2 then set true
    NEQ,
    /// If src1 > src2 then set true
    GT,
    /// If src1 < src2 then set true
    LT,
    /// If src1 >= src2 then set true
    GTE,
    /// If src1 <= src2 then set true
    LTE,
    /// If equaly_bool(A special register for storing last equality result) == true then jmp
    JMPE,

    /// No operate
    NOP,
    /// For memory
    ALOC,
    /// Illegal opcode
    IGL,
}
```
