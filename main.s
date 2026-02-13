	.file	"main.9b5cc465332d101d-cgu.0"
	.section	.text._ZN3std2rt10lang_start17hd525fffb7dfb32ddE,"ax",@progbits
	.hidden	_ZN3std2rt10lang_start17hd525fffb7dfb32ddE
	.globl	_ZN3std2rt10lang_start17hd525fffb7dfb32ddE
	.p2align	4
	.type	_ZN3std2rt10lang_start17hd525fffb7dfb32ddE,@function
_ZN3std2rt10lang_start17hd525fffb7dfb32ddE:
	.cfi_startproc
	pushq	%rax
	.cfi_def_cfa_offset 16
	movl	%ecx, %r8d
	movq	%rdx, %rcx
	movq	%rsi, %rdx
	movq	%rdi, (%rsp)
	leaq	.Lanon.dc508296685befcb133aa148f88e1d77.0(%rip), %rsi
	movq	%rsp, %rdi
	callq	*_ZN3std2rt19lang_start_internal17h74b643a2cc7fe3b4E@GOTPCREL(%rip)
	popq	%rcx
	.cfi_def_cfa_offset 8
	retq
.Lfunc_end0:
	.size	_ZN3std2rt10lang_start17hd525fffb7dfb32ddE, .Lfunc_end0-_ZN3std2rt10lang_start17hd525fffb7dfb32ddE
	.cfi_endproc

	.section	".text._ZN3std2rt10lang_start28_$u7b$$u7b$closure$u7d$$u7d$17hed403516632d8d8bE","ax",@progbits
	.p2align	4
	.type	_ZN3std2rt10lang_start28_$u7b$$u7b$closure$u7d$$u7d$17hed403516632d8d8bE,@function
_ZN3std2rt10lang_start28_$u7b$$u7b$closure$u7d$$u7d$17hed403516632d8d8bE:
	.cfi_startproc
	pushq	%rax
	.cfi_def_cfa_offset 16
	movq	(%rdi), %rdi
	callq	_ZN3std3sys9backtrace28__rust_begin_short_backtrace17h524443a768056252E
	xorl	%eax, %eax
	popq	%rcx
	.cfi_def_cfa_offset 8
	retq
.Lfunc_end1:
	.size	_ZN3std2rt10lang_start28_$u7b$$u7b$closure$u7d$$u7d$17hed403516632d8d8bE, .Lfunc_end1-_ZN3std2rt10lang_start28_$u7b$$u7b$closure$u7d$$u7d$17hed403516632d8d8bE
	.cfi_endproc

	.section	.text._ZN3std3sys9backtrace28__rust_begin_short_backtrace17h524443a768056252E,"ax",@progbits
	.p2align	4
	.type	_ZN3std3sys9backtrace28__rust_begin_short_backtrace17h524443a768056252E,@function
_ZN3std3sys9backtrace28__rust_begin_short_backtrace17h524443a768056252E:
	.cfi_startproc
	pushq	%rax
	.cfi_def_cfa_offset 16
	callq	*%rdi
	#APP
	#NO_APP
	popq	%rax
	.cfi_def_cfa_offset 8
	retq
.Lfunc_end2:
	.size	_ZN3std3sys9backtrace28__rust_begin_short_backtrace17h524443a768056252E, .Lfunc_end2-_ZN3std3sys9backtrace28__rust_begin_short_backtrace17h524443a768056252E
	.cfi_endproc

	.section	".text._ZN4core3ops8function6FnOnce40call_once$u7b$$u7b$vtable.shim$u7d$$u7d$17hdb66c14479d38a03E","ax",@progbits
	.p2align	4
	.type	_ZN4core3ops8function6FnOnce40call_once$u7b$$u7b$vtable.shim$u7d$$u7d$17hdb66c14479d38a03E,@function
_ZN4core3ops8function6FnOnce40call_once$u7b$$u7b$vtable.shim$u7d$$u7d$17hdb66c14479d38a03E:
	.cfi_startproc
	pushq	%rax
	.cfi_def_cfa_offset 16
	movq	(%rdi), %rdi
	callq	_ZN3std3sys9backtrace28__rust_begin_short_backtrace17h524443a768056252E
	xorl	%eax, %eax
	popq	%rcx
	.cfi_def_cfa_offset 8
	retq
.Lfunc_end3:
	.size	_ZN4core3ops8function6FnOnce40call_once$u7b$$u7b$vtable.shim$u7d$$u7d$17hdb66c14479d38a03E, .Lfunc_end3-_ZN4core3ops8function6FnOnce40call_once$u7b$$u7b$vtable.shim$u7d$$u7d$17hdb66c14479d38a03E
	.cfi_endproc

	.section	.text._ZN4main4main17h7d177e22a494942bE,"ax",@progbits
	.hidden	_ZN4main4main17h7d177e22a494942bE
	.globl	_ZN4main4main17h7d177e22a494942bE
	.p2align	4
	.type	_ZN4main4main17h7d177e22a494942bE,@function
_ZN4main4main17h7d177e22a494942bE:
	.cfi_startproc
	retq
.Lfunc_end4:
	.size	_ZN4main4main17h7d177e22a494942bE, .Lfunc_end4-_ZN4main4main17h7d177e22a494942bE
	.cfi_endproc

	.section	.text.main,"ax",@progbits
	.globl	main
	.p2align	4
	.type	main,@function
main:
	.cfi_startproc
	pushq	%rax
	.cfi_def_cfa_offset 16
	movq	%rsi, %rcx
	movslq	%edi, %rdx
	leaq	_ZN4main4main17h7d177e22a494942bE(%rip), %rax
	movq	%rax, (%rsp)
	leaq	.Lanon.dc508296685befcb133aa148f88e1d77.0(%rip), %rsi
	movq	%rsp, %rdi
	xorl	%r8d, %r8d
	callq	*_ZN3std2rt19lang_start_internal17h74b643a2cc7fe3b4E@GOTPCREL(%rip)
	popq	%rcx
	.cfi_def_cfa_offset 8
	retq
.Lfunc_end5:
	.size	main, .Lfunc_end5-main
	.cfi_endproc

	.type	.Lanon.dc508296685befcb133aa148f88e1d77.0,@object
	.section	.data.rel.ro..Lanon.dc508296685befcb133aa148f88e1d77.0,"aw",@progbits
	.p2align	3, 0x0
.Lanon.dc508296685befcb133aa148f88e1d77.0:
	.asciz	"\000\000\000\000\000\000\000\000\b\000\000\000\000\000\000\000\b\000\000\000\000\000\000"
	.quad	_ZN4core3ops8function6FnOnce40call_once$u7b$$u7b$vtable.shim$u7d$$u7d$17hdb66c14479d38a03E
	.quad	_ZN3std2rt10lang_start28_$u7b$$u7b$closure$u7d$$u7d$17hed403516632d8d8bE
	.quad	_ZN3std2rt10lang_start28_$u7b$$u7b$closure$u7d$$u7d$17hed403516632d8d8bE
	.size	.Lanon.dc508296685befcb133aa148f88e1d77.0, 48

	.ident	"rustc version 1.93.0 (254b59607 2026-01-19)"
	.section	".note.GNU-stack","",@progbits
