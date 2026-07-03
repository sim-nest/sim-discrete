# sim-lib-discrete

In one line: one front door to the whole discrete-math family, where you switch on only the parts you need.

## What it gives you

This is the single entry point that gathers the discrete-math pieces under one roof. Rather than tracking several separate packages, you reach for this one and turn on exactly the capabilities your task calls for: the shared calculation core, the graph tools, the counting and enumeration helpers, the component-analysis instruments, and the ranking adapters. Anything you leave switched off simply does not come along, so a small job stays small and a large one pulls in only what it truly uses. On top of that gathering, this crate supplies the wiring that lets these tools show up as live operations inside the running SIM system, plus a browsable index so people can find what is on offer.

## Why you will be glad

- You depend on one clearly named package and opt into features, instead of juggling many pieces by hand.
- Leaving unused parts switched off keeps builds lean and your footprint honest.
- The same tools become available both as a plain library and as live operations inside the running system.

## Where it fits

This crate is the assembly point of the discrete-math family in SIM. It holds no heavy computation of its own; each real capability lives in its own focused package, and this one simply presents them together behind clean switches. That arrangement gives newcomers a single obvious place to start while letting every underlying piece stay small, independent, and separately testable.
