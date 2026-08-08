; Like tit-for-tat but defects on round 0 instead of cooperating.
; Expect(self, opp) = [(1,0), (0,1), (1,0), (0,0)]
    LOAD R0, ROUND
    CMPI R0, #0
    JEQ defect
    LOAD R0, LASTOPP
    CMP R0, DEFECT
    JEQ defect
    PLAY COOPERATE

defect:
    PLAY DEFECT
