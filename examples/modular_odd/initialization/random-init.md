**Random initialization**

This initialization submodel distributes turtles randomly over the landscape.

```
// Initialization
; included from initialization/center-init.md
create-turtles 10 [
  setxy random-pxcor random-pycor
  pen-down
]
reset-ticks
```