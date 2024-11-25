# This is the default target for "make"
.PHONY: 2048
2048:
	cargo run obj/2048.obj 2>/dev/null

# Does not show anything other than the game on stdout
# make run program=obj/2048.obj
.PHONY: run
run:
	cargo run $(program) 2>/dev/null

# This will show stderr as well, so you should redirect it to another tty to see the game
# make run program=obj/2048.obj 2>/dev/ttys000
.PHONY: debug
debug:
	cargo run $(program)

.PHONY: build
build:
	cargo build

# Builds all *.asm files in asm/ and moves the *.obj and generated *.sym
# files to the obj/ directory
ASM_FILES := $(wildcard asm/*.asm)
.PHONY: asm
asm:
	@mkdir -p obj
	@for file in $(ASM_FILES); do \
		base=$$(basename $$file .asm); \
		echo "Assembling $$file..."; \
		lc3as $$file; \
		mv asm/$$base.obj obj/$$base.obj; \
		mv asm/$$base.sym obj/$$base.sym; \
	done

.PHONY: clean
clean:
	cargo clean && rm obj/*.sym && rm obj/*.obj

.PHONY: test
test:
	cargo test
