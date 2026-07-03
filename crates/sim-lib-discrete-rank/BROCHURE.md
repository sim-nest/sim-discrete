# sim-lib-discrete-rank

In one line: turn a discrete object into a position and a grade so you can place it, score it, and compare it.

## What it gives you

This crate connects the discrete-math family to SIM's ranking machinery. It takes concrete objects -- a set of chosen items, an ordering, a small network, a on-or-off signal, a bundle of flags -- and gives each one a clear place within the full space of possibilities. From there you can measure how far apart two objects are, so "similar" and "very different" become numbers you can act on. It also compiles a grade from an object's own structure: how tightly a network holds together, how heavy its cheapest connecting skeleton is, how spread out its component pattern is. The result is a single, comparable score drawn straight from meaningful traits rather than an arbitrary label.

## Why you will be glad

- You can position, sort, and compare discrete objects consistently instead of inventing an ad hoc scoring rule each time.
- Distance measures let you find nearest matches and cluster related items on solid ground.
- Grades come from real structural traits, so a score means something you can explain and defend.

## Where it fits

This crate is the single agreed bridge between the discrete-math family in SIM and the shared ranking library. It is the one place allowed to lean on that library, and the reliance runs strictly one way, so the ranking side never has to know these details. Keeping the connection in one clear spot protects both sides and gives the rest of the system a tidy door to ranked discrete structure.
