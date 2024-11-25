# LC3 VM

This is an implementation of a [LC3 VM](https://www.jmeiners.com/lc3-vm/)
written in Rust.

![](https://private-user-images.githubusercontent.com/889301/389672388-85096552-271e-4a94-8573-4025557d4093.png?jwt=eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpc3MiOiJnaXRodWIuY29tIiwiYXVkIjoicmF3LmdpdGh1YnVzZXJjb250ZW50LmNvbSIsImtleSI6ImtleTUiLCJleHAiOjE3MzI1NjI1MzcsIm5iZiI6MTczMjU2MjIzNywicGF0aCI6Ii84ODkzMDEvMzg5NjcyMzg4LTg1MDk2NTUyLTI3MWUtNGE5NC04NTczLTQwMjU1NTdkNDA5My5wbmc_WC1BbXotQWxnb3JpdGhtPUFXUzQtSE1BQy1TSEEyNTYmWC1BbXotQ3JlZGVudGlhbD1BS0lBVkNPRFlMU0E1M1BRSzRaQSUyRjIwMjQxMTI1JTJGdXMtZWFzdC0xJTJGczMlMkZhd3M0X3JlcXVlc3QmWC1BbXotRGF0ZT0yMDI0MTEyNVQxOTE3MTdaJlgtQW16LUV4cGlyZXM9MzAwJlgtQW16LVNpZ25hdHVyZT00NTQwMzZlMmYzZWU4NWI0MGE0NDVhZmQ0ZWQyODY2ZDM2MTE5MmRjM2Q5OWQzNTM5MzE5ZGUzYzJlZjY2OTgwJlgtQW16LVNpZ25lZEhlYWRlcnM9aG9zdCJ9.fOKG99WaVm34_d1FTHt0pn0PB5-SMlP4oF5NL_xOXKE)

## Running

Clone the repository
* `git clone git@github.com:dsocolobsky/lc3-vm.git && cd lc3-vm`

To run the default game 2048 just do:
* `make run`

To run whatever obj file you want you can do:
* `make run program=obj/rogue.obj`

This by default redirects stderr to /dev/null, for debug information you can do something like:
* `make debug program=obj/2048.obj 2>/dev/ttys000`
Where `/dev/ttys000` is whatever tty you have open, to see the debug output.

If you have installed the lc3 assembler `lc3as` you can build whatever `.asm` files you have in `asm/` directory
with:
* `make asm`
