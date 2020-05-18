to setup
  create-turtles 10 [
    pen-down
  ]
  reset-ticks
end

to go
  ask turtles [
    right (random 90) - 45
    forward 1
  ]
  tick
end

@#$#@#$#@
BUTTON
10
10
80
50
NIL
setup
NIL
1
T
OBSERVER
NIL
NIL
NIL
NIL
1

BUTTON
90
10
160
50
NIL
go
T
1
T
OBSERVER
NIL
NIL
NIL
NIL
1


GRAPHICS-WINDOW
220
10
-1
-1
-1
-1
5.0
1
10
1
1
1
0
1
1
1
0
100
0
100
1
1
1
ticks
30.0

@#$#@#$#@
## WHAT IS IT?

## HOW IT WORKS

## HOW TO USE IT

@#$#@#$#@
default
true
0
Polygon -7500403 true true 150 5 40 250 150 205 260 250
@#$#@#$#@
NetLogo 6.1.0
@#$#@#$#@
@#$#@#$#@
@#$#@#$#@
@#$#@#$#@
@#$#@#$#@
default
0.0
-0.2 0 0.0 1.0
0.0 1 1.0 0.0
0.2 0 0.0 1.0
link direction
true
0
Line -7500403 true 150 150 90 180
Line -7500403 true 150 150 210 180
@#$#@#$#@
0
@#$#@#$#@

