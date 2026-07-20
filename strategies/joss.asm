; Like tit-for-tat but randomly defects ~10% of the time even when opponent cooperated.
; Expect: action=Cooperate
    LOAD R0, LASTOPP
    CMP R0, DEFECT
    JEQ defect
    RDRAND R0
    CMPI R0, #102
    JGT defect
    PLAY COOPERATE

defect:
    PLAY DEFECT
