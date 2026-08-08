; Win-stay, lose-shift: repeat last action if payoff was good (>=3), switch otherwise.
; Expect(self, opp) = [(0,0), (0,1), (1,1), (0,0), (0,0)]
    LOAD R0, ROUND
    CMPI R0, #0
    JEQ cooperate

    LOAD R0, LASTPAYOFFSELF
    CMPI R0, #2
    JGT win

    LOAD R0, LASTSELF
    CMP R0, DEFECT
    JEQ cooperate
    JMP defect

win:
    LOAD R0, LASTSELF
    CMP R0, DEFECT
    JEQ defect

cooperate:
    PLAY COOPERATE
defect:
    PLAY DEFECT
