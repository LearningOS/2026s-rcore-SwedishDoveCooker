## 编程作业

基本和提示的思路一样, 在syscall和TaskManagerInner里面修改了内容, 扩展了一个简单的map用于存储syscall次数, 然后在TaskManager里面添加相关方法用于增加和读取次数。

## 简答作业

1. 

   ``````
   [kernel] PageFault in application, bad addr = 0x0, bad instruction = 0x804003a4, kernel killed it.
   [kernel] IllegalInstruction in application, kernel killed it.
   [kernel] IllegalInstruction in application, kernel killed it.
   ``````

   ``````
   [rustsbi] RustSBI version 0.3.0-alpha.2, adapting to RISC-V SBI v1.0.0
   [rustsbi] Implementation     : RustSBI-QEMU Version 0.2.0-alpha.2
   ``````

2. 

   1. 
      sp指向内核为这次Trap handle所分配的栈, 栈上存放着一个TrapContext, 里面保存了用户程序的所有寄存器; _restore可以用于syscall之后的恢复, 也可以用于程序运行前的trap初始化。

   2. 

      ``````
      ld t0, 32*8(sp)
      csrw sstatus, t0
      ``````

      从内核栈加载保存的 sstatus 到 t0 并将 t0 的值写回 sstatus; sstatus `SPP` 等字段给出 Trap 发生之前 CPU 处在哪个特权级（S/U）等信息。

      

      ``````
      ld t1, 33*8(sp)
      csrw sepc, t1
      ``````

      从内核栈加载保存的 sepc 到 t1 并将 t1 的值写回 sepc; 当 Trap 是一个异常的时候，spec 记录 Trap 发生之前执行的最后一条指令的地址。

      

      ``````
      ld t2, 2*8(sp)
      csrw sscratch, t2
      ``````

      从内核栈加载保存的 sp 到 t2 并将 t2 的值写回 sscratch; `sscratch` CSR 为中转寄存器。

   3. `tp(x4)` 除非我们手动出于一些特殊用途使用它，否则一般也不会被用到。

      sp(x2) 在后面保存, 因为我们要基于它来找到每个寄存器应该被保存到的正确的位置

      ````
      mv a0, sp
      ````

   4. 
      执行完成后 sp 指向用户栈, sscratch指向内核栈

   5. 
      
      sret指令, 执行完成后cpu会跳转到sepc的地址执行

      sret会完成以下功能：

      - CPU 会将当前的特权级按照 `sstatus` 的 `SPP` 字段设置为 U 或者 S ；
      - CPU 会跳转到 `sepc` 寄存器指向的那条指令，然后继续执行。

   6. 
      执行完成后 sp 指向内核栈, sscratch指向用户栈

   7. 
      ecall指令