# LC3 VM

This is an implementation of a [LC3 VM](https://www.jmeiners.com/lc3-vm/)
written in Rust.

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
