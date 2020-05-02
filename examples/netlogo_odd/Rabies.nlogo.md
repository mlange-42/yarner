# From ODD to NetLogo


This document explores the creation of a [NetLogo](https://ccl.northwestern.edu/netlogo/) model
from it's own ODD model description (Grimm et al. 2006, 2010).
Processed with `outline`, the document produces a runnable file of the described model
for the famous NetLogo modelling environment.

All (!) code of the model is in this document, including some NetLogo-specific boilerplate code,
which can be found in the [Appendix](#appendix).

**Content**

* [Overview](#overview)
* [Design concepts](#design-concepts)
* [Details](#details)
* [Simulation experiments, parameters and analysis](#simulation-experiments-parameters-and-analysis)
* [Appendix](#appendix)

## Overview

### Purpose

The model simulates rabies in a spatially structured fox population.
The original purpose of **the model** is to teach veterinarians in
epidemiological modelling, in a hands-on course where they build this model.

The purpose of **this document** is to test the potential of Literate Programming to derive (simple)
models from their own ODD model description (the code blocks therein).

### Entities, state variables, and scales

The only entities in the model are fox families, represented by patches (the NetLogo term for grid cells).
Each patch has the following state variables:

``` - Patches
; define patch variables
patches-own [
  state               ; disease state of patch (see globals, default = EMPTY)
  infected-neighbours ; number of infected neighbours of patch
  ticks-to-death      ; ticks remaining until death of infected
  dispersal-tick      ; tick-of year (month) of dispersal
]

```

The `state` represents the presence of a fox family, and its epidemiological status: 

``` - Globals
; define global variables
globals [
  ; constants for patch (cell) states
  EMPTY  ; Empty patch
  S      ; Susceptible foxes
  I      ; Infected foxes
  R      ; Recovered / vaccinated foxes
]

```

State names are just identifiers. Internally, they are integers:

``` - Setup globals
set EMPTY 0
set S 1
set I 2
set R 3
```

The world is a torus of 100 x 100 patches (see Appendix, [Interface tab](#interface-tab)).
Patches are assumed to be 1 x 1 km.

The model proceeds in discrete monthly steps (ticks).

### Process overview and scheduling

Processes in the model are dispersal (abstracting away reproduction),
infection, and aging of infection (to death).
The processes are executed on each tick sequentially, in the following order:

``` - Go
; one model step (called by button)
to go
  ==> Assign dispersal...
  disperse

  infect-patches
  age-infection

  update-patch-color
  tick
end

```

## Design concepts

Fox population dynamics emerge from reproduction (abstracted into stochastic dispersal),
rabies-induces mortality and limited space to occupy.

Rabies dynamics emerge from stochastic disease transmission between neighbors,
and the death of foxes after a certain time span of infection.

## Details

### Initialization

The model is initialized by setting all patches to susceptible (S).
Then, one randomly selected patch is infected.

``` - Setup
; clears and sets up the model
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
  
  update-patch-color
  reset-ticks         ; reset model ticks (and tell NetLogo that the simulation can start!)
end

```

### Input data

The model does not use any input data.

### Submodels

Submodels are described in the order of their execution:

``` - Submodels
==> Dispersal...
==> Disease transmission...
==> Disease course...
```

#### Dispersal

At the start of each year, the month of dispersal for each fox family's offspring is determined.

``` - Assign dispersal
if ticks mod 12 = 0 [
  assign-dispersal
]
```

The dispersal tick is a random month of the year,
selected in the range given by parameters `start-dispersal` and `length-dispersal` (in months).

The selected month is stored in the patch state variable `dispersal-tick`.

``` - Dispersal
; assign step of year for dispersal
to assign-dispersal
  ask patches with [ state != EMPTY ] [
    set dispersal-tick (ticks + start-dispersal + random length-dispersal)
  ]
end

```

In every model step, all fox families are checked if it is currently their `dispersal-tick`,
and disperse if this is the case (i.e. once per year).

For each dispersing fox family, all empty patches in radius `dispersal-radius` are collected.
If their number exceeds parameter `num-offspring`,
`num-offspring` of these patches are selected randomly and set to susceptible (i.e. occupied).
If the number of collected empty patches is equal to or smaller than `num-offspring`,
they are all set to susceptible.

``` - Dispersal
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

```

#### Disease transmission

Rabies is transmitted stochastically between neighboring fox families (8 neighbors).

First, the number of infected neighbors is calculated for each patch
and stored in patch variable `infected-neighbours`.
Then, each susceptible fox family is infected with a probability according
function `infection-prob` (see below).

``` - Disease transmission
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

==> Infection...
==> Infection probability...
```

The infection probability is calculated according the Reed-Frost model
from the number of infected neighbors:

``` - Infection probability
; calculates the infection probability for a patch / fox family
to-report infection-prob
  report 1 - (1 - beta) ^ infected-neighbours
end

```

#### Infection

Upon infection, the fox family's `state` is set to infected,
and the ticks remaining to their rabies-induced death (`ticks-to-death`) are set.

``` - Infection
; infects a patch / fox family
to infect
  set state I
  set ticks-to-death ticks-infected
end

```

#### Disease course

For each infected fox family, their `ticks-to-death` are counted down each step.
When they reach zero, the patch is set to empty (i.e. death of the foxes).

``` - Disease course
; ages the infection and removes dead foxes
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

### Simulation experiments

Simulation experiments are left to the user.

### Observation

The model can be observed through a colored grid view. Colors are updates after every model step:

``` - Other functions
; updates the color of all patches
to update-patch-color
  ask patches [
    ifelse state = EMPTY [ set pcolor white ]
    [ ifelse state = S [ set pcolor blue ]
    [ ifelse state = I [ set pcolor red ]
    [ set pcolor green ] ] ]
  ]
end

```

### Analysis

Analysis is also left to the user

### Parameters

Parameters can be adjusted freely by the user using the sliders on the [Interface tab](#interface-tab).

Recommended parameters are:
* `start-dispersal = 7` (August)
* `length-dispersal = 2` (2 months)
* `dispersal-radius = 2.5` (2.5 km)
* `num-offspring = 4`
* `beta = 0.2`
* `ticks-infected = 2` (2 months)

## Appendix

### Code structure

In the NetLogo Code tab, the code is structured as follows:

``` - Code tab
==> Globals...
==> Patches...
==> Setup...
==> Go...
==> Submodels...
==> Other functions...
```

### File structure

A NetLogo file is composed of sections, delimited by mysterious `@#$#@#$#@`.
The content of each section can be found somewhere in this document.
Further details about the `.nlogo` file atructure can be found in [NetLogo's Hithub repository](https://github.com/NetLogo/NetLogo/wiki/File-(.nlogo)-and-Widget-Format).

```
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

Here, we just insert a short description in NetLogo's Markdown syntax:

``` - Info tab
## WHAT IS IT?

A simple individual-based model of rabies in fox populations.

The speciality of this model is that it was created from it's own ODD model description,
using Literate Programming and the tool `outline`.

## HOW IT WORKS

This is described in the documentation this model was created from.

## HOW TO USE IT

Just play with the model. But also read the documentation which is the base of the model.
```

### Interface tab

Setting up the user interface in NetLogo programmatically is a bit inconvenient.

We add two buttons, some sliders, and set up the world and graphics window,
using `outline`'s "meta variables" feature:

``` - Interface tab
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

``` - Button @{name} @{left} @{top} @{right} @{bottom} @{forever}
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

``` - Slider @{name} @{left} @{top} @{right} @{bottom} @{min} @{max} @{step} @{value}
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

``` - GraphicsWindow @{width} @{height} @{wrap_x} @{wrap_y} @{left} @{top} @{patch_size} @{fps} @{on_tick}
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

``` - Turtle shapes
default
true
0
Polygon -7500403 true true 150 5 40 250 150 205 260 250
```

### Link shapes

And the default link shape:

``` - Link shapes
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

Last but not least, the file's NetLogo version is required.

``` - NetLogo version
NetLogo 6.1.0
```
