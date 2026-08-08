; Like tit-for-tat but only retaliates after two consecutive defections.
; Expect(self, opp) = [(0,1), (0,1), (1,0), (0,0), (0,0)]
    LOADI R1, #0
    LOAD R0, LASTOPP
    LOAD R2, SCRATCH[R1]
    STORE SCRATCH[R1], R0
    CMP R0, DEFECT
    JNE cooperate
    CMP R2, DEFECT
    JNE cooperate
    PLAY DEFECT

cooperate:
    PLAY COOPERATE
