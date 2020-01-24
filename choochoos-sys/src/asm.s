.global Yield

Yield:
	swi #0
	bx  lr

	.global Exit

Exit:
	swi #1
	bx  lr

	.global MyTid

MyTid:
	swi #2
	bx  lr

	.global MyParentTid

MyParentTid:
	swi #3
	bx  lr

	.global Create

Create:
	swi #4
	bx  lr

	.global Send

Send:
	swi #5
	bx  lr

	.global Receive

Receive:
	swi #6
	bx  lr

	.global Reply

Reply:
	swi #7
	bx  lr
