;;! target = "aarch64"

(module
    (func (result i64)
        (i64.const 1)
     	(i64.const 0)
    	(i64.add)
    )
)
;;    0:	 fd7bbfa9             	stp	x29, x30, [sp, #-0x10]!
;;    4:	 fd030091             	mov	x29, sp
;;    8:	 fc030091             	mov	x28, sp
;;    c:	 ff2300d1             	sub	sp, sp, #8
;;   10:	 fc030091             	mov	x28, sp
;;   14:	 890300f8             	stur	x9, [x28]
;;   18:	 300080d2             	mov	x16, #1
;;   1c:	 e00310aa             	mov	x0, x16
;;   20:	 00000091             	add	x0, x0, #0
;;   24:	 ff230091             	add	sp, sp, #8
;;   28:	 fc030091             	mov	x28, sp
;;   2c:	 fd7bc1a8             	ldp	x29, x30, [sp], #0x10
;;   30:	 c0035fd6             	ret	
