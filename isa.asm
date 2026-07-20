#bankdef default
{
    #bits 8
    #addr 0
    #size 0x100
    #outp 0
}

#ruledef {
    RDRAND {rd: reg}            => 0x80 @ rd

    CMPI {ra: reg}, #{imm: i8}  => 0x51 @ ra @ imm
    TEST {rd: reg}, #{imm: i8}  => 0x52 @ rd @ imm

    JEQ {label: u8}             => 0x60 @ label

    PLAY COOPERATE               => 0x70
    PLAY DEFECT                  => 0x71
}

#ruledef reg {
    R0 => 0x0`8
    R1 => 0x1`8
    R2 => 0x2`8
    R3 => 0x3`8
}
