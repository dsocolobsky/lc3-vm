.ORIG x3000

MAIN
    AND R2, R2, #0
    ADD R2, R2, #3
LOOP
    ADD R2, R2, #-1
    BRp LOOP
FINI
    HALT
.END
