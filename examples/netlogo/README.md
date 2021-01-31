# NetLogo + ODD Example

This example demonstrates how to use Yarner to create NetLogo models from documentation structured according the ODD protocol (Overview, Design, Details; [Grimm et al. 2006, 2010](#references)). 

For help on the Literate Programming syntax, see the [User Guide](https://mlange-42.github.io/yarner/).

To create the NetLogo model, run the following command in the current directory:

```
> yarner
```

**Reverse mode**

This project is set up to enable Yarner's reverse mode. In the generated model, you will find comment lines that delineate code blocks. Do not delete or modify these lines. Except this limitation, you can modify the model inside Netlogo, and afterwards play back changes into the documentation sources with

```
> yarner --reverse
```

To allow for the reverse mode, NetLogo's `__includes` feature is used. The actual code is not inside the model's main file `Model.nlogo`, but in a separate file `Code.nls`. To edit the code inside NetLogo, us the dropdown `Included files` in the Code tab.

To get clean code output without block labels, run yarner with option `--clean`:

```
> yarner --clean
```

**Content**

* [Overview](#overview)
* [Design concepts](#design-concepts)
* [Details](#details)
  * [Initialization](#initialization)
  * [Input data](#input-data)
  * [Submodels](#submodels)
* [References](#references)
* [Appendix](#appendix)

## Overview

### Purpose

The purpose of this simple model is to demonstrate how to use Yarner to create NetLogo models.

### Entities, state variables, and scales

The only entities in the model are walkers. The only state variable of each walker besides NetLogo's built-in state variables is its speed.

Internally, walkers further have a color and a position.

```netlogo
;- Entities
breed [ walkers walker ]

walkers-own [
  speed
]
```

### Process overview and scheduling

The only process of the model is a random walk of walkers.

```netlogo
;- Go
to go
  random-walk
  tick
end
```

## Design concepts

Walkers make a random walk and draw a line along their path.

## Details

### Initialization

The model is initialized with 100 walkers at random locations.

```netlogo
;- Setup
to setup
  create-walkers 100 [
    setxy random-xcor random-ycor
    set speed 0.5 + random-float 0.5
    pen-down
  ]
  reset-ticks
end
```

### Input data

The model uses no input date.

### Submodels

#### Random walk

Walkers do a correlated random walk.

In each model step, each walker turns a maximum of 45°. Turn angle is uniformly distributed between -45° and +45°.

After turning, the walker steps forward according to its speed.

```netlogo
;- Submodels
to random-walk
  ask walkers [
    right (random 90) - 45
    forward speed
  ]
end
```

## References

Grimm V, Berger U, Bastiansen F, Eliassen S, Ginot V, Giske J, Goss-Custard J, Grand T, Heinz S, Huse G, Huth A, Jepsen JU, Jørgensen C, Mooij WM, Müller B, Pe’er G, Piou C, Railsback SF, Robbins AM, Robbins MM, Rossmanith E, Rüger N, Strand E, Souissi S, Stillman RA, Vabø R, Visser U, DeAngelis DL. 2006. **A standard protocol for describing individual-based and agent-based models.** Ecological Modelling 198:115-126. 

Grimm V, Berger U, DeAngelis DL, Polhill G, Giske J, Railsback SF. 2010. **The ODD protocol: a review and first update.** Ecological Modelling 221: 2760-2768

## Appendix

The @[Appendix](appendix.md) is used to create the `.nlogo` with UI elements, turtle shapes, etc.

The content of the code tab is created by simply drawing together all blocks shown above:

```netlogo
;- file:Code.nls
; ==> Entities.
; ==> Setup.
; ==> Go.
; ==> Submodels.
```
