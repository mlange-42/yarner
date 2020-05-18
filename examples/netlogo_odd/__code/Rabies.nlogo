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
  state               ; disease state of patch (see globals, default = EMPTY)
  infected-neighbours ; number of infected neighbours of patch
  ticks-to-death      ; ticks remaining until death of infected
  dispersal-tick      ; tick-of year (month) of dispersal
]

; clears and sets up the model
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

  update-patch-color
  reset-ticks         ; reset model ticks (and tell NetLogo that the simulation can start!)
end

; one model step (called by button)
to go
  if ticks mod 12 = 0 [
    assign-dispersal
  ]
  disperse

  infect-patches
  age-infection

  update-patch-color
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
      set state S ; no dispersal of infected!
    ]
  ]
end

; calculates infections for all patches / fox families
to infect-patches
  ask patches [                    ; reset number of infected neighbours
    set infected-neighbours 0
  ]
  ask patches with [ state = I ] [ ; iterate over infected patches
    ask neighbors [                ; increase counter of all neighbours
      set infected-neighbours infected-neighbours + 1
    ]
  ]
  ask patches with [ state = S ] [
    if random-float 1 < infection-prob [
      infect
    ]
  ]
end

; infects a patch / fox family
to infect
  set state I
  set ticks-to-death ticks-infected
end

; calculates the infection probability for a patch / fox family
to-report infection-prob
  report 1 - (1 - beta) ^ infected-neighbours
end

; ages the infection and removes dead foxes
to age-infection
  ask patches with [ state = I ] [
    if ticks-to-death = 0 [
      set state EMPTY
    ]
    set ticks-to-death ticks-to-death - 1
  ]
end

; updates the color of all patches
to update-patch-color
  ask patches [
    ifelse state = EMPTY [ set pcolor white ]
    [ ifelse state = S [ set pcolor blue ]
    [ ifelse state = I [ set pcolor red ]
    [ set pcolor green ] ] ]
  ]
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


SLIDER
10
120
200
140
start-dispersal
start-dispersal
0
11
7
1
1
NIL
HORIZONTAL

SLIDER
10
160
200
170
length-dispersal
length-dispersal
1
12
2
1
1
NIL
HORIZONTAL

SLIDER
10
200
200
210
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
270
num-offspring
num-offspring
0
10
4
1
1
NIL
HORIZONTAL


SLIDER
10
320
200
330
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
370
ticks-infected
ticks-infected
1
10
2
1
1
NIL
HORIZONTAL


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

A simple individual-based model of rabies in fox populations.

The speciality of this model is that it was created from it's own ODD model description,
using Literate Programming and the tool `yarner`.

## HOW IT WORKS

This is described in the documentation this model was created from.

## HOW TO USE IT

Just play with the model. But also read the documentation which is the base of the model.
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

