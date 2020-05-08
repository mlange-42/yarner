# Modular ODD with Literate Programming

For help on the Literate Programming syntax, see the [README](https://github.com/mlange-42/outline/blob/master/README.md) on GitHub.

**Content**

* [Overview](#overview)
* [Design concepts](#design-concepts)
* [Details](#details)
* [Simulation experiments, parameters and analysis](#simulation-experiments-parameters-and-analysis)
* [Appendix](#appendix)

## Overview

### Purpose

@{{[overview/purpose.md](overview/purpose.md)}}

### Entities, state variables, and scales

@{{[overview/entities-scales.md](overview/entities-scales.md)}}

### Process overview and scheduling

The only process in the model is turtle movement.

```
// Go
to go
  ==> Turtle movement...
  tick
end

```

## Design concepts

Turtles are initialised and move in different ways, depending on which submodels are included in this document (called transclusion). Transclusions are links in curly braces after an @:

```
// Transclusion syntax
@{{[Title](target.md)}}
```

## Details

### Initialization

```
// Setup
to setup
  ==> Initialization...
end

```

@{{[initialization/random-init.md](initialization/random-init.md)}}

### Input data

The model uses no input data.

### Submodels

#### Turtle movement

@{{[movement/straight-walk.md](movement/straight-walk.md)}}

----

## Appendix 

Literate Programming entry point (required for technical reasons):

```
==> Netlogo file structure...
```

@{{[appendix/appendix.md](appendix/appendix.md)}}