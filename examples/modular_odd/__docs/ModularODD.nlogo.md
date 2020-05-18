# Modular ODD with Literate Programming

For help on the Literate Programming syntax, see the [README](https://github.com/mlange-42/yarner/blob/master/README.md) on GitHub.

**Content**

* [Overview](#overview)
* [Design concepts](#design-concepts)
* [Details](#details)
* [Simulation experiments, parameters and analysis](#simulation-experiments-parameters-and-analysis)
* [Appendix](#appendix)

## Overview

### Purpose

The purpose of the model is to demonstrate modular ODD Protocols through Literate Programming using file transclusions.

This document and the model code are assembled from an ODD with code blocks, where ODD sections and submodels are pulled together from different files.

### Entities, state variables, and scales

The only entities in the model are NetLogo turtles. 

Turtle state variables are the usual NetLogo builtin variables position (`xcor`, `ycor`) and `heading`.

Turtles live in a rectangular world of size 100 by 100 units.

```nlogo
// Create world
==> GraphicsWindow at @{220} @{10} size @{100} @{100} wrap @{} @{} patch-size @{}.
```

### Process overview and scheduling

The only process in the model is turtle movement.

```nlogo
// Go
to go
  ==> Turtle movement.
  tick
end

```

## Design concepts

Turtles are initialised and move in different ways, depending on which submodels are included in this document (called transclusion). Transclusions are links in curly braces after an @:

```nlogo
// Transclusion syntax
@{{[Title](target.md)}}
```

## Details

### Initialization

```nlogo
// Setup
to setup
  ==> Initialization.
end

```

**Random initialization**

This initialization submodel distributes turtles randomly over the landscape.

```nlogo
// Initialization
; included from initialization/center-init.md
create-turtles 10 [
  setxy random-pxcor random-pycor
  pen-down
]
reset-ticks
```

### Input data

The model uses no input data.

### Submodels

#### Turtle movement

**Straight walk**

Turtles walk straight in a random direction.

```nlogo
// Turtle movement
; included from movement/straight-walk.md
ask turtles [
  forward 1
]
```

----

## Appendix 

Literate Programming entry point (required for technical reasons):

```nlogo
==> Netlogo file structure.
```

### Code structure

In the NetLogo Code tab, the code is structured as follows:

```nlogo
// Code tab
==> Setup.
==> Go.
```

### File structure

A NetLogo file is composed of sections, delimited by mysterious `@#$#@#$#@`.
The content of each section can be found somewhere in this document.
Further details about the `.nlogo` file atructure can be found in [NetLogo's Hithub repository](https://github.com/NetLogo/NetLogo/wiki/File-(.nlogo)-and-Widget-Format).

```nlogo
// Netlogo file structure
==> Code tab.
@#$#@#$#@
==> Interface tab.
@#$#@#$#@
==> Info tab.
@#$#@#$#@
==> Turtle shapes.
@#$#@#$#@
==> NetLogo version.
@#$#@#$#@
@#$#@#$#@
@#$#@#$#@
@#$#@#$#@
@#$#@#$#@
==> Link shapes.
@#$#@#$#@
0
@#$#@#$#@

```

### Info tab

Here, we just insert a short description in NetLogo's Markdown syntax:

```nlogo
// Info tab
## WHAT IS IT?

## HOW IT WORKS

## HOW TO USE IT

```

### Interface tab

Setting up the user interface in NetLogo programmatically is a bit inconvenient.

We add two buttons, some sliders, and set up the world and graphics window,
using `yarner`'s "meta variables" feature:

```nlogo
// Interface tab
==> Button @{setup} @{10} @{10} @{80} @{50} @{NIL}.
==> Button @{go} @{90} @{10} @{160} @{50} @{T}.

==> Create world.
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
// GraphicsWindow at @{left} @{top} size @{width} @{height} wrap @{wrap_x:1} @{wrap_y:1} patch-size @{patch_size:5.0}
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
1
1
1
ticks
30.0

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

And the default link shape:

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

Last but not least, the file's NetLogo version is required.

```nlogo
// NetLogo version
NetLogo 6.1.0
```
