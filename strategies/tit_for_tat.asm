; Expect: action=Cooperate
    LOAD R0, LASTOPP
    CMP R0, DEFECT
    JEQ defect
    PLAY COOPERATE

defect:
    PLAY DEFECT
