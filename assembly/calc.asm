;promt
        LDA #'T'
        STA $F001
        LDA #'h'
        STA $F001
        LDA #'e'
        STA $F001
        LDA #' '
        STA $F001
        LDA #'a'
        STA $F001
        LDA #'n'
        STA $F001
        LDA #'s'
        STA $F001
        LDA #'w'
        STA $F001
        LDA #'e'
        STA $F001
        LDA #'r'
        STA $F001
        LDA #':'
        STA $F001
        LDA #' '
        STA $F001
        LDA #'2'
        STA $F001
        LDA #' '
        STA $F001
        LDA #'+'
        STA $F001
        LDA #' '
        STA $F001
        LDA #'3'
        STA $F001
        LDA #' '
        STA $F001
        LDA #'='
        STA $F001
        LDA #' '
        STA $F001

        ;result
        LDA #2
        CLC
        ADC #3
        ADC #'0'
        STA $F001
        LDA #10
        STA $F001
; rather unoptimal printing of the result, tho it seems era appropriate...
