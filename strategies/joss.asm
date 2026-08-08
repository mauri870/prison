; Like tit-for-tat but randomly defects ~10% of the time even when opponent cooperated.
; Seed: 1
; Expect(self, opp) = [(0,1), (1,0)]
    LOAD R0, LASTOPP
    CMP R0, DEFECT
    JEQ defect
    RDRAND R0
    LOADI R1, #10
    MOD R0, R1
    CMPI R0, #0
    JEQ defect
    PLAY COOPERATE

defect:
    PLAY DEFECT
