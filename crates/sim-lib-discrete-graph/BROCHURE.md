# sim-lib-discrete-graph

In one line: model anything as a network of connections and find out how the pieces link, reach, and cost.

## What it gives you

Whenever your problem is really a set of things joined by links -- roads between towns, dependencies between tasks, friendships between people -- this crate turns it into a graph you can question. You can walk it to see what is connected to what, check whether the whole thing hangs together or splits into islands, find the cheapest set of links that still joins everything, and trace the shortest route between any two points. Just as useful, many answers arrive with a certificate: a small piece of evidence you can independently re-check to confirm the result is genuinely correct rather than taken on trust.

## Why you will be glad

- You get connectivity, minimum-cost spanning structures, and shortest paths from one consistent toolkit instead of stitching libraries together.
- Certificate-producing checks let you verify an answer, which matters when a decision rides on it.
- Heavy path and reachability work reuses the shared algebra core, so behavior stays predictable across problem sizes.

## Where it fits

This crate is the graph surface of the discrete-math family in SIM. It leans on the algebra spine for the demanding all-pairs and reachability computations rather than duplicating them, and it stays free of any downstream concern like music or ranking. That focus keeps it a clean, dependable place to reach for whenever a task is naturally shaped as a network.
