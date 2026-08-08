; Cooperate until opponent defects, then punish with N defections (N = total opponent defections),
; followed by two cooperative rounds to signal reconciliation.
; Expect(self, opp) = [(0,0), (0,1), (1,0), (0,0), (0,1), (1,0), (1,0), (0,0), (0,0)]
    LOADI R0, #2
    LOAD R1, SCRATCH[R0]
    CMPI R1, #0
    JEQ check_punish
    LOADI R3, #1
    SUB R1, R3
    STORE SCRATCH[R0], R1
    JMP cooperate

check_punish:
    LOADI R0, #1
    LOAD R1, SCRATCH[R0]
    CMPI R1, #0
    JNE punish_action

    LOAD R0, LASTOPP
    CMP R0, DEFECT
    JNE cooperate

    LOADI R0, #0
    LOAD R1, SCRATCH[R0]
    INC R1
    STORE SCRATCH[R0], R1
    LOADI R0, #1
    STORE SCRATCH[R0], R1

punish_action:
    LOADI R3, #1
    SUB R1, R3
    STORE SCRATCH[R0], R1
    CMPI R1, #0
    JNE defect
    LOADI R0, #2
    LOADI R1, #2
    STORE SCRATCH[R0], R1

defect:
    PLAY DEFECT
cooperate:
    PLAY COOPERATE
