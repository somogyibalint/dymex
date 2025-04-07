::: mermaid
---
config:
  layout: elk
  look: handDrawn
  theme: dark
---
flowchart TB
  expr@{ shape: doc, label: "(2.0×pi × exp(-x×x)) / max(1.0 + sqrt(y), 0)" }
  var0@{ shape: cyl, label: "x" }
  var1@{ shape: cyl, label: "y" }
  S0{" \/ "}
  S1{" \* "}
  S2{" \* "}
  S3[ 2 ]
  S4[[ π ]]
  S5[\ Exp /]
  S6{" \- "}
  S7{" \* "}
  S8( x )
  S9( x )
  S10> Max ]
  S11{" \+ "}
  S12[ 1 ]
  S13[\ Sqrt /]
  S14( y )
  S15[ 0 ]
  S2-->S3
  S2-->S4
  S1-->S2
  S7-->S8
  S7-->S9
  S6-->S7
  S5-->S6
  S1-->S5
  S0-->S1
  S11-->S12
  S13-->S14
  S11-->S13
  S10-->S11
  S10-->S15
  S0-->S10
:::
