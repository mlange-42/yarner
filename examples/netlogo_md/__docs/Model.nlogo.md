# Setting up a NetLogo file

This document explains the setup of a minimal NetLogo model file,
and, processed with `yarner`, produces the described file from the documentation.

First, we explain the actual [NetLogo code](#code). 
Next, we see the [file structure](#file-structure) of a `.nlogo` file. 
Then the remaining code, not so important parts of the NetLogo file, is presented.
Particularly the [Info tab](#info-tab), [Interface tab](#interface-tab)
and some other required code.

## Code

Here we insert two simple procedures that will be called from buttons (see [Interface tab](#interface-tab))

First, we need to set up the world. We do this in `setup`.

```nlogo
// Code tab
to setup
  clear-all
  create-turtles 10 [
    pen-down
  ]
  reset-ticks
end

```

Next, some fancy things shall happen in `go`.

```nlogo
// Code tab
to go
  ask turtles [
    ==> Per-turtle action.
  ]
  tick
end
```

In every model step, each turtle will then do the following:

```nlogo
// Per-turtle action
forward 1
right (random 90) - 45
```

## File structure

A NetLogo file is composed of sections, delimited by mysterious `@#$#@#$#@`.
Each section is explained and shown as code below.
Details for `.nlogo` files can be found in [NetLogo's Hithub repository](https://github.com/NetLogo/NetLogo/wiki/File-(.nlogo)-and-Widget-Format).

```nlogo
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

## Info tab

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

## Interface tab

Setting up the user interface in NetLogo is a bit inconvenient.
First, we need to set up the world's attributes:

```nlogo
// Interface tab
GRAPHICS-WINDOW
210
10
647
448
-1
-1
13.0
1
10
1
1
1
0
1
1
1
-16
16
-16
16
1
1
1
ticks
30.0

```

Then, we add two buttons:

```nlogo
// Interface tab
==> Button @{setup} @{16} @{13} @{79} @{46} @{NIL}.
==> Button @{go} @{85} @{13} @{148} @{46} @{T}.
```

The actual button code is presented in the [Appendix](#appendix)

## Turtle shapes

We need to define the default shape for turtles:

```nlogo
// Turtle shapes
default
true
0
Polygon -7500403 true true 150 5 40 250 150 205 260 250
```

# Link shapes

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

## NetLogo version

```nlogo
// NetLogo version
NetLogo 6.1.0
```

# Appendix

Code for a Button in NetLogo:

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
