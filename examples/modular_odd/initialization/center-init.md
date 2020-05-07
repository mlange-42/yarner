**Central initialization**

This initialization submodel places all turtles in the center of the world.

``` - Initialization
; included from initialization/center-init.md
create-turtles 10 [
  setxy  max-pxcor / 2  max-pycor / 2 
  pen-down
]
reset-ticks
```