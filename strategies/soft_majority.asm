; Cooperate if opponent has cooperated at least as often as defected, otherwise defect.
; Expect(self, opp) = [(0,0), (0,1), (0,1), (1,0), (0,0)]
    LOADI R0, #0
    LOAD R1, ROUND
    CMPI R1, #0
    JEQ cooperate

    LOAD R1, LASTOPP
    LOAD R2, SCRATCH[R0]
    CMP R1, DEFECT
    JEQ decrement

    INC R2
    JMP store

decrement:
    LOADI R3, #1
    SUB R2, R3

store:
    STORE SCRATCH[R0], R2
    CMPI R2, #0
    JLT defect

cooperate:
    PLAY COOPERATE
defect:
    PLAY DEFECT
