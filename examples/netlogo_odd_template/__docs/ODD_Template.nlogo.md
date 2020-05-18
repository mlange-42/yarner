# ODD Template

For help on the Literate Programming syntax, see [README](https://github.com/mlange-42/yarner/blob/master/README.md).

**Content**

* [Overview](#overview)
* [Design concepts](#design-concepts)
* [Details](#details)
* [Simulation experiments, parameters and analysis](#simulation-experiments-parameters-and-analysis)
* [Appendix](#appendix)

## Overview

### Purpose

### Entities, state variables, and scales

### Process overview and scheduling

```nlogo
// Go
to go
  ==> Random walk...
  tick
end

```

## Design concepts

## Details

### Initialization

```nlogo
// Setup
to setup
  create-turtles 10 [
    pen-down
  ]
  reset-ticks
end

```

### Input data

### Submodels

#### Random walk

```nlogo
// Random walk
ask turtles [
  right (random 90) - 45
  forward 1
]
```

## Appendix

### Code structure

In the NetLogo Code tab, the code is structured as follows:

<!-- Here, you put together all top-level code parts from above -->

```nlogo
// Code tab
==> Setup...
==> Go...
```

### File structure

A NetLogo file is composed of sections, delimited by mysterious `@#$#@#$#@`.
The content of each section can be found somewhere in this document.
Further details about the `.nlogo` file atructure can be found in [NetLogo's Hithub repository](https://github.com/NetLogo/NetLogo/wiki/File-(.nlogo)-and-Widget-Format).

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
==> Button @{setup} @{10} @{10} @{80} @{50} @{NIL}...
==> Button @{go} @{90} @{10} @{160} @{50} @{T}...

==> GraphicsWindow at @{220} @{10} size @{100} @{100} wrap @{} @{} ps @{} fps @{} tick @{}...
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
// GraphicsWindow at @{left} @{top} size @{width} @{height} wrap @{wrap_x:1} @{wrap_y:1} ps @{patch_size:5.0} fps @{fps:30.0} tick @{on_tick:1}
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
