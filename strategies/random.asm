; Cooperate or defect at random with equal probability.
; Expect: seed=42 action=Cooperate
    RDRAND R0
    TEST R0, #1
    JEQ cooperate
    PLAY DEFECT

cooperate:
    PLAY COOPERATE
