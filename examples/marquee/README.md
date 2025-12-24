# marquee

A silly tool for testing AXM packets.

Provides the following:

1. Where am I? (where-the-fuck-i-am) returning "x y z heading" which can be used in the
   commands below
2. painted letterboard marquee (letterboard x y z heading "your test here")
3. painted letter marquee (painted x y z heading "your test here")
4. circle of tyre stacks (circle x y z heading radius count)
5. Pre-fab "signal" (collection of objects) placement (x y z heading)

Ultimate plan: this is a testing ground for silly ideas relating to layout objects, and
a may later turn into 2 or 3 tools to build layouts quickly. The marquee functionality
is really a different idea.

The marquee functionality would be extremely helpful to split out potentially as a
utility. Which would require:

1. A config file
2. Support for both letterboards and painted text at the same time (it's hard coded for
   one or the other)
3. Support for multiple locations through the configuration file
