# sim-lib-discrete-graph

In one line: model anything as a network of connections and find out how the pieces link, reach, and cost.

## What it gives you

Whenever your problem is really a set of things joined by links -- roads between towns, dependencies between tasks, friendships between people -- this crate turns it into a graph you can question. You can walk it to see what is connected to what, check whether the whole thing hangs together or splits into islands, find the cheapest set of links that still joins everything, trace the shortest route between any two points, pair unequal collections while forbidding illegal matches, optimize a choice through staged layers, and align sequences under explicit window, gap, endpoint, and memory policies. Just as useful, these optimization answers arrive with evidence you can independently re-check instead of taking a result on trust.

## Why you will be glad

- You get connectivity, minimum-cost spanning structures, and shortest paths from one consistent toolkit instead of stitching libraries together.
- Minimum-cost assignment handles unequal sides, forbidden pairs, and voice-order constraints without factorial permutation search.
- Layered paths and edit/DTW alignment preserve deterministic ties and offer full certificates or rolling-memory score-only evidence.
- Shared controls bound cell/edge work and memory, honor deadlines and cancellation, and reject NaN or infinite costs before they can corrupt an optimum.
- Certificate-producing checks let you verify an answer, which matters when a decision rides on it.
- Heavy path and reachability work reuses the shared algebra core, so behavior stays predictable across problem sizes.

## Where it fits

This crate is the graph and finite dynamic-programming surface of the discrete-math family in SIM. It leans on the algebra spine for demanding all-pairs and reachability computations rather than duplicating them, and it stays free of downstream concerns like music or ranking. Music, statistics, and analysis libraries can therefore compose the same assignment and alignment evidence without growing private algorithm copies.
