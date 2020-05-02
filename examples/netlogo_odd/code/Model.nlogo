to setup
  clear-all
  create-turtles 10 [
    pen-down
  ]
  reset-ticks
end

to go
  ask turtles [
    forward 1
    right (random 90) - 45
  ]
  tick
end
@#$#@#$#@
BUTTON
16
13
79
46
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
85
13
148
46
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
500
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
0
0
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

(a general understanding of what the model is trying to show or explain)

## HOW IT WORKS

(what rules the agents use to create the overall behavior of the model)

## HOW TO USE IT

(how to use the model, including a description of each of the items in the Interface tab)
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
