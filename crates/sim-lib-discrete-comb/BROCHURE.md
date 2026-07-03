# sim-lib-discrete-comb

In one line: count and list every way to arrange or choose things, and give each arrangement its own exact number.

## What it gives you

This crate answers the everyday "how many ways" questions and then goes further. It counts arrangements, selections, and orderings exactly, with no rounding and no ceiling, so even enormous totals stay precise. When you want the arrangements themselves and not just the tally, it can hand them to you one at a time without building the whole pile in memory. Best of all, it gives every possible arrangement a unique position number and lets you go both directions: name a position and get the exact arrangement, or hand it an arrangement and learn its position. That pairing turns a vast space of possibilities into something you can index, sample, and step through on demand.

## Why you will be glad

- Exact counts hold their precision no matter how large the answer grows, so you never silently lose accuracy.
- You can generate arrangements lazily and stop whenever you like, keeping memory use modest.
- The position-to-arrangement mapping lets you jump straight to the item you want without listing everything before it.

## Where it fits

This crate is the counting and enumeration corner of the discrete-math family in SIM. Its unique-numbering helpers are the natural bridge to the ranking tools, yet it stays self-contained and does not reach into them. That keeps it a plain, reusable source of exact combinatorial answers for anything else in the system that needs to count or walk possibilities.
