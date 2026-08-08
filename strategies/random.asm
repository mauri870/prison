; Cooperate or defect at random with equal probability.
; Seed: 42
; Expect(self, opp) = [(0,0)]
    RDRAND R0
    TEST R0, #1
    JEQ cooperate
    PLAY DEFECT

cooperate:
    PLAY COOPERATE
