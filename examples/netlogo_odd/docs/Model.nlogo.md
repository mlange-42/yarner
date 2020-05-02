# From ODD to NetLogo


This document explores the creation of a NetLogo model from an ODD,
Processed with `outline`, the document produces a runnable file of the described model.

## Overview

### Purpose

### Entities, state variables, and scales

The in the model are fox families, represented by patches. Each patch has the following state variables:

```nlogo
// Patches
; define patch variables
patches-own [
  state               ; disease state of patch (see globals, default = 0)
  infected-neighbours ; number of infected neighbours of patch
  ticks-to-death      ; ticks remaining until death of infected
  dispersal-tick      ; tick-of year (month) of dispersal
]
```

The `state` represents the presence of a fax familiy, and its epidemiological status: 

```nlogo
// Globals
; define global variables
globals [
  ; constants for patch (cell) states
  EMPTY  ; Empty patch
  S      ; Susceptible foxes
  I      ; Infected foxes
  R      ; Recovered / vaccinated foxes
]
```

States are just identifiers. Internally, they are just integers:

```nlogo
// Setup globals
set EMPTY 0
set S 1
set I 2
set R 3
```

### Process overview and scheduling

Processes are executed in the following order per tick:

```nlogo
// Go
; one model step (called by button)
to go
  ==> Assign dispersal...
  disperse

  infect-patches
  age-infection
  update-patches
  tick
end
```

## Design concepts

## Details

### Initialization

The model is initialized by setting all patches to susceptible (S).
One randomly selected patch is infected.

```nlogo
// Setup
to setup
  clear-all

  ==> Setup globals...
  
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
```

### Input data

The model does not use any input data.

### Submodels

```nlogo
// Submodels
==> Dispersal...
==> Disease transmission...
==> Disease course...
```

#### Dispersal

At the start of each year, the month of dispersal for each fox family is determined
and saved in state variable `dispersal-tick`.

```nlogo
// Assign dispersal
if ticks mod 12 = 0 [
  assign-dispersal
]
```

```nlogo
// Dispersal
; assign step of year for dispersal
to assign-dispersal
  ask patches with [ state != EMPTY ] [
    set dispersal-tick (ticks + start-dispersal + random length-dispersal)
  ]
end
```

In every model step, all fox families are checked if it is currently their `dispersal-tick`, and disperse if so.

```nlogo
// Dispersal
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
```

#### Disease transmission

```nlogo
// Disease transmission
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

==> Infection...
==> Infection probability...
```

```nlogo
// Infection probability
to-report calc-infection-prob
  report 1 - (1 - beta) ^ infected-neighbours
end
```

#### Infection

Upon infection, the fox family's `state` is set to infected,
and the ticks remaining to their rabies-induced death (`ticks-to-death`) are set.

```nlogo
// Infection
to infect
  set state I
  set ticks-to-death ticks-infected
end
```

#### Disease course

For each infected fox family, their `ticks-to-death` are counted down each step.
When they reach zero, the patch is set to empty (i.e. death of the foxes).

```nlogo
// Disease course
to age-infection
  ask patches with [ state = I ] [
    if ticks-to-death = 0 [
      set state EMPTY
    ]
    set ticks-to-death ticks-to-death - 1
  ]
end
```

## Simulation experiments, parameters and analysis

### Simulation experiments, analysis

Simulation experiments are left to the user.

### Observation

The model can be observed through a colored grid view. Colors are updates after every model step.

```nlogo
// Other functions
; updates the color of all patches
to update-patches
  ask patches [
    ifelse state = EMPTY [ set pcolor white ]
    [ ifelse state = S [ set pcolor blue ]
    [ ifelse state = I [ set pcolor red ]
    [ set pcolor green ] ] ]
  ]
end
```

### Parameters

Parameters can be adjusted freely by the user. Recommended parameters are:

[TODO]

### Analysis

## Appendix

### Code structure

```nlogo
// Code tab
==> Globals...
==> Patches...
==> Setup...
==> Go...
==> Submodels...
==> Other functions...
```

### File structure

A NetLogo file is composed of sections, delimited by mysterious `@#$#@#$#@`.
Each section is explained and shown as code below.
Details for `.nlogo` files can be found in [NetLogo's Hithub repository](https://github.com/NetLogo/NetLogo/wiki/File-(.nlogo)-and-Widget-Format).

```nlogo
==> Code tab...
@#$#@#$#@
==> Interface tab...
@#$#@#$#@
==> Info tab...
@#$#@#$#@
==> Turtle shapes...
@#$#@#$#@
==> NetLogo version...
@#$#@#$#@
@#$#@#$#@
@#$#@#$#@
@#$#@#$#@
@#$#@#$#@
==> Link shapes...
@#$#@#$#@
0
@#$#@#$#@

```

### Info tab

Here, we just insert a short description template, in NetLogo's syntax:

```nlogo
// Info tab
## WHAT IS IT?

(a general understanding of what the model is trying to show or explain)

## HOW IT WORKS

(what rules the agents use to create the overall behavior of the model)

## HOW TO USE IT

(how to use the model, including a description of each of the items in the Interface tab)
```

### Interface tab

Setting up the user interface in NetLogo is a bit inconvenient.

We add two buttons, some sliders, and set up the graphics window:

```nlogo
// Interface tab
==> Button @{setup} @{10} @{10} @{80} @{50} @{NIL}...
==> Button @{go} @{90} @{10} @{160} @{50} @{T}...

==> Slider @{start-dispersal} @{10} @{120} @{200} @{140} @{0} @{11} @{1} @{7}...
==> Slider @{length-dispersal} @{10} @{160} @{200} @{170} @{1} @{12} @{1} @{2}...
==> Slider @{dispersal-radius} @{10} @{200} @{200} @{210} @{1} @{10} @{0.5} @{2.5}...

==> Slider @{num-offspring} @{10} @{260} @{200} @{270} @{0} @{10} @{1} @{4}...

==> Slider @{beta} @{10} @{320} @{200} @{330} @{0} @{1} @{0.01} @{0.2}...
==> Slider @{ticks-infected} @{10} @{360} @{200} @{370} @{1} @{10} @{1} @{2}...

==> GraphicsWindow @{100} @{100} @{1} @{1} @{220} @{10} @{5.0} @{30.0} @{1}...
```

The actual code for a Button in NetLogo looks like this:

```nlogo
// Button @{name} @{left} @{top} @{right} @{bottom} @{forever}
BUTTON
@{left}
@{top}
@{right}
@{bottom}
NIL
@{name}
@{forever}
1
T
OBSERVER
NIL
NIL
NIL
NIL
1

```

The actual code for a Slider in NetLogo looks like this:

```nlogo
// Slider @{name} @{left} @{top} @{right} @{bottom} @{min} @{max} @{step} @{value}
SLIDER
@{left}
@{top}
@{right}
@{bottom}
@{name}
@{name}
@{min}
@{max}
@{value}
@{step}
1
NIL
HORIZONTAL

```

The actual code for the Graphics window looks like this:

```nlogo
// GraphicsWindow @{width} @{height} @{wrap_x} @{wrap_y} @{left} @{top} @{patch_size} @{fps} @{on_tick}
GRAPHICS-WINDOW
@{left}
@{top}
-1
-1
-1
-1
@{patch_size}
1
10
1
1
1
0
@{wrap_x}
@{wrap_y}
1
0
@{width}
0
@{height}
@{on_tick}
@{on_tick}
1
ticks
@{fps}

```

### Turtle shapes

We need to define the default shape for turtles:

```nlogo
// Turtle shapes
default
true
0
Polygon -7500403 true true 150 5 40 250 150 205 260 250
```

### Link shapes

And finally, the default link shape:

```nlogo
// Link shapes
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
```

### NetLogo version

```nlogo
// NetLogo version
NetLogo 6.1.0
```
