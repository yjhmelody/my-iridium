# A register-based VM

# 未完成，大概坑了!!!

## 操作码


## 二进制布局

- header
- read-only data
- executable data

### Header

用于验证的首部，4字节的魔数，总长64字节，后面60个暂时保留。
```rust
pub const PIE_HEADER_PREFIX: [u8; 4] = [45, 50, 49, 45];
pub const PIE_HEADER_LENGTH: usize = 64;
```

### Read-only Data

只读段用于存储一些常量，比如字符串
```
hello: .asciiz 'Hello'
```

- hello 是一个标签
- .asciiz 声明了常量类型
- 'Hello' 是常量本身
- 写入只读段后，字符串末尾填充0，最终为`[72, 101, 108, 108, 111, 0]`，所有字符串用UTF-8表示
- 所有常量紧凑连接

当引用了常量 比如 PRTS @hello，汇编器在第一趟pass做了以下工作
- 找到`hello:`并且检查它没有出现在符号表
- 检查到了`.asciiz`指示，这意味着将下一个操作数视为以null结尾的字符串
- 解析了Hello，删除了单引号
- 在符号表中记录了该字符串常量开始的位置
- 要检索字符串常量，我们在符号表中进行查找，并从那里开始读取字节，直到我们遇见0
- PRTS指令知道查看只读部分

一些使用规则
- 用户必须在 .data 段 声明
- 所有字符串默认为UTF-8，以0结尾
- 声明字符串常量的格式为 my_string: .asciiz '<string>'
- 在 .code 段，用户使用 @my_string 作为操作数

## REPL

详见 `repl` 和 `bin`

## 汇编器

详见 `assembler`

## 字符串

### UTF-8 说明

- 0xxx xxxx    A single-byte US-ASCII code (from the first 127 characters)
- 110x xxxx    One more byte follows
- 1110 xxxx    Two more bytes follow
- 1111 0xxx    Three more bytes follow

## 符号表

## 并发并行

基本原理：创建新的系统线程，并传递新的`VM`，让它执行`vm.run()`直到返回。就像linux一样，给每个线程设置自己的PID。

## 基准测试

对 VM 的一些指令的性能做最基本的测试

## 远程登录