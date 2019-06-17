target extended-remote :3333

# print demangled symbols
set print asm-demangle on

# set backtrace limit to not have infinite backtrace loops
set backtrace limit 32

# detect panics
break rust_begin_unwind

# break at main().
break main

# Un-comment if you add something that uses semihosting.
#monitor arm semihosting enable

# Load the program onto the chip.
load

# start the program but immediately halt the processor
stepi
