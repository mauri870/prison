; Cooperate until the opponent defects once, then defect forever.
; Expect: action=Cooperate
    LOADI R0, #0
    LOAD R1, SCRATCH[R0]
    TEST R1, #1
    JNE defect
    LOAD R0, LASTOPP
    CMP R0, DEFECT
    JEQ trigger
    PLAY COOPERATE

trigger:
    LOADI R0, #0
    LOADI R1, #1
    STORE SCRATCH[R0], R1
defect:
    PLAY DEFECT
