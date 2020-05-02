; define global variables
globals [
  ; constants for patch (cell) states
  EMPTY  ; Empty patch
  S      ; Susceptible foxes
  I      ; Infected foxes
  R      ; Recovered / vaccinated foxes
]
; define patch variables
patches-own [
  state               ; disease state of patch (see globals, default = 0)
  infected-neighbours ; number of infected neighbours of patch
  ticks-to-death      ; ticks remaining until death of infected
  dispersal-tick      ; tick-of year (month) of dispersal
]
to setup
  clear-all

  set EMPTY 0
  set S 1
  set I 2
  set R 3

  ; set al patches to susceptible
  ask patches [
    set state S
  ]

  ; infect one patch
  ask one-of patches [
    infect
  ]

  update-patches   ; update patch color
  reset-ticks      ; reset model ticks (and tell NetLogo that the simulation can start!)
end
; one model step (called by button)
to go
  if ticks mod 12 = 0 [
    assign-dispersal
  ]
  disperse

  infect-patches
  age-infection
  update-patches
  tick
end
; assign step of year for dispersal
to assign-dispersal
  ask patches with [ state != EMPTY ] [
    set dispersal-tick (ticks + start-dispersal + random length-dispersal)
  ]
end
; dispersal of offspring
to disperse
  ask patches with [ state != EMPTY and dispersal-tick = ticks ] [
    let candidates other patches
                   in-radius dispersal-radius
                   with [ state = EMPTY ]
    let num-candidates num-offspring
    if count candidates < num-candidates [
      set num-candidates  count candidates
    ]
    ask n-of num-candidates candidates [
      set state S ; no dispersal of infectedf
    ]
  ]
end
to infect-patches
  ask patches [     ; reset number of infected neighbours
    set infected-neighbours 0
  ]
  ask patches with [ state = I ] [ ; iterate over infected patches
    ask neighbors [                ; increase counter of all neighbours
      set infected-neighbours infected-neighbours + 1
    ]
  ]
  ask patches with [ state = S ] [
    if random-float 1 < calc-infection-prob [
      infect
    ]
  ]
end

to infect
  set state I
  set ticks-to-death ticks-infected
end
to-report calc-infection-prob
  report 1 - (1 - beta) ^ infected-neighbours
end
to age-infection
  ask patches with [ state = I ] [
    if ticks-to-death = 0 [
      set state EMPTY
    ]
    set ticks-to-death ticks-to-death - 1
  ]
end
; updates the color of all patches
to update-patches
  ask patches [
    ifelse state = EMPTY [ set pcolor white ]
    [ ifelse state = S [ set pcolor blue ]
    [ ifelse state = I [ set pcolor red ]
    [ set pcolor green ] ] ]
  ]
end
@#$#@#$#@
GRAPHICS-WINDOW
220
10
733
524
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

SLIDER
10
120
200
153
start-dispersal
start-dispersal
0
11
7.0
1
1
NIL
HORIZONTAL

SLIDER
10
160
200
193
length-dispersal
length-dispersal
1
12
2.0
1
1
NIL
HORIZONTAL

SLIDER
10
200
200
233
dispersal-radius
dispersal-radius
1
10
2.5
0.5
1
NIL
HORIZONTAL

SLIDER
10
260
200
293
num-offspring
num-offspring
0
10
4.0
1
1
NIL
HORIZONTAL

SLIDER
10
320
200
353
beta
beta
0
1
0.2
0.01
1
NIL
HORIZONTAL

SLIDER
10
360
200
393
ticks-infected
ticks-infected
1
10
2.0
1
1
NIL
HORIZONTAL

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
