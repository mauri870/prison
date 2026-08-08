; Cooperate on round 0, then mirror the opponent's last move.
; Expect(self, opp) = [(0,0), (0,1), (1,0), (0,0)]
    LOAD R0, LASTOPP
    CMP R0, DEFECT
    JEQ defect
    PLAY COOPERATE

defect:
    PLAY DEFECT
