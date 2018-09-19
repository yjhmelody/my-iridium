# 汇编器

## 具体格式

[label] opcode [operand1 [operand2 [operand3]]]

说明：
- label 可选，声明格式为 `name:`，使用格式为 `@name`, 可以作为操作数使用
- opcode 必需，后面跟0-3个操作数或label
- 操作数分为 源操作数，目的操作数。通常源操作数在前，目的操作数在后
- directives，即指示，控制汇编器做一定的事情，格式为`.name ...`

### 举例

load $1 #100
add $1 $2 $1
label1: add $1 $2 $3
