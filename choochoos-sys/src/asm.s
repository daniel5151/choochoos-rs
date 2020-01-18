.global Yield
Yield:
    swi #0
    bx lr

.global Exit
Exit:
    swi #1
    bx lr

.global MyParentTid
MyParentTid:
    swi #2
    bx lr

.global MyTid
MyTid:
    swi #3
    bx lr

.global Create
Create:
    swi #4
    bx lr
