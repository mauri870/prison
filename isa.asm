; prison ISA
;
; 4 general-purpose registers: R0, R1, R2, R3 (signed 8-bit, wrapping arithmetic)
; 256 bytes of per-match scratch memory (reset each match)
; 256 bytes of per-tournament persistent memory
; All jumps take a label; targets are resolved at assemble time.
; Programs execute from the top each round and must end with PLAY.
; Hitting HALT or exhausting the instruction budget cooperates by default.

#bankdef default
{
    #bits 8
    #addr 0
    #size 0x100
    #outp 0
}

#ruledef {
    NOP                         => 0x00
    HALT                        => 0x01

    MOV   {rd: reg}, {rs: reg}  => 0x10 @ rd @ rs
    LOADI {rd: reg}, #{imm: i8} => 0x11 @ rd @ imm

    LOAD  {rd: reg}, SCRATCH[{rs: reg}]  => 0x20 @ rd @ rs
    STORE SCRATCH[{rs: reg}], {rd: reg}  => 0x21 @ rs @ rd
    LOAD  {rd: reg}, MEMORY[{rs: reg}]   => 0x22 @ rd @ rs
    STORE MEMORY[{rs: reg}], {rd: reg}   => 0x23 @ rs @ rd

    LOAD {rd: reg}, LASTSELF       => 0x30 @ rd @ 0x0`8
    LOAD {rd: reg}, LASTOPP        => 0x30 @ rd @ 0x1`8
    LOAD {rd: reg}, ROUND          => 0x30 @ rd @ 0x2`8
    LOAD {rd: reg}, SCORESELF      => 0x30 @ rd @ 0x3`8
    LOAD {rd: reg}, SCOREOPP       => 0x30 @ rd @ 0x4`8
    LOAD {rd: reg}, OPPID          => 0x30 @ rd @ 0x5`8
    LOAD {rd: reg}, LASTPAYOFFSELF => 0x30 @ rd @ 0x6`8

    ADD {rd: reg}, {rs: reg}    => 0x40 @ rd @ rs
    SUB {rd: reg}, {rs: reg}    => 0x41 @ rd @ rs
    INC {rd: reg}               => 0x42 @ rd
    AND {rd: reg}, {rs: reg}    => 0x43 @ rd @ rs
    OR  {rd: reg}, {rs: reg}    => 0x44 @ rd @ rs
    XOR {rd: reg}, {rs: reg}    => 0x45 @ rd @ rs

    CMP  {ra: reg}, {rb: reg}   => 0x50 @ ra @ rb
    CMPI {ra: reg}, #{imm: i8}  => 0x51 @ ra @ imm
    CMP  {ra: reg}, {c: consts} => 0x51 @ ra @ c
    TEST {rd: reg}, #{imm: i8}  => 0x52 @ rd @ imm

    JEQ {label: u8}             => 0x60 @ label
    JNE {label: u8}             => 0x61 @ label
    JLT {label: u8}             => 0x62 @ label
    JGT {label: u8}             => 0x63 @ label
    JMP {label: u8}             => 0x64 @ label

    PLAY COOPERATE               => 0x70
    PLAY DEFECT                  => 0x71

    RDRAND {rd: reg}            => 0x80 @ rd
}

; Four general-purpose signed 8-bit registers
#ruledef reg {
    R0 => 0x0`8
    R1 => 0x1`8
    R2 => 0x2`8
    R3 => 0x3`8
}

#ruledef consts {
    COOPERATE => 0x0`8  ; 0
    DEFECT    => 0x1`8  ; 1
}
