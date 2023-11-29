---
sidebar_position: 5
---

import CodeSnippet from '@site/src/components/CodeSnippet';
import Core from '@substrate/docs/examples/examples/core.rs?snippet';

# Simulation

## Simulators

Substrate aims to make it easy to plug and play different simulators. The way it does this is by providing a
minimal interface that each simulator must implement, defined in the [`Simulator`](https://api.substratelabs.io/substrate/simulation/trait.Simulator.html) trait.

In essence, a simulator Substrate plugin simply needs to specify a function that takes in a set of inputs and options and returns a set of outputs or an error. The types of the inputs, outputs, options, and errors are entirely user designed, providing flexibility to the plugin writer. A simulator also needs to have an associated schema (see the [SCIR chapter](./scir.md) for more details), which defines the format in which it should be provided SCIR libraries that represent the schematic that needs to be simulated.

### Analyses

Once a simulator is defined, a set of supported analyses can be defined using the [`Analysis`](https://api.substratelabs.io/substrate/simulation/trait.SupportedBy.html) and [`SupportedBy`](https://api.substratelabs.io/substrate/simulation/trait.SupportedBy.html) traits, which essentially convert an analysis (e.g. transient, AC, op) to inputs that the `Simulator` trait can understand and reformats the outputs of the simulator plugin into the output expected from the analysis.

### Options

Simulators can also provide a set of options that allow users to modify the behavior of the simulator by carefully designing the interface for their options type. An example of this is the Spectre plugin's [`Options`](https://api.substratelabs.io/spectre/struct.Options.html) type, which allows users to specify includes, saved currents/voltages, and more. Additional supported options, especially ones that should be simulator-portable, can be defined using the [`SimOption`](https://api.substratelabs.io/substrate/simulation/options/trait.SimOption.html) trait.

### Saved data

Simulators can also specify what data can be saved from a testbench using the 
[`FromSaved`](https://api.substratelabs.io/substrate/simulation/data/trait.FromSaved.html) and 
[`Save`](https://api.substratelabs.io/substrate/simulation/data/trait.Save.html) traits. The `Save` 
trait modifies the simulator options to keep track of what data needs to be saved and returns a key for 
accessing that data. The `FromSaved` trait then takes this key and uses it to retrieve the associated data 
from the simulation output after the simulation has run. The simulator plugin writer will need to 
store keys in the options and propagate them to the simulation output so that data can be retrieved correctly.

Simulators should generally support saving currents and voltages for nodes and terminals in Substrate and SCIR formats.
