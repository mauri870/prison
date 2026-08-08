; Cooperate until the opponent defects once, then defect forever.
; Expect(self, opp) = [(0,0), (0,1), (1,0), (1,0)]
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
