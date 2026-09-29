# ISSUE #2990 by @jabalsad: Feature request: print shell-compatible variable evaluations
Hi there,

When running `just --evaluate`, or e.g. on a specific file `just --evaluate --file env.just`, to get the variables printed in such a way that they can be sourced / evaluated in the context of a shell.

For a justfile:
```
VAR := "foo"
export BAR := "bar"
```

Current behaviour:
```
$ just --evaluate
VAR := "foo"
BAR := "bar"
```

Desired behaviour (perhaps with an optional argument like `--shell`, or an entirely different command `--shell-eval`)
```
$ just --evaluate
VAR="foo"
export BAR="bar"
```

This allows me to run:
```
$ eval `just --evaluate`
```

--- @bluefing:
You could use sed as a workaround maybe 

```shell
>just --evaluate | sed -r 's/[ ]+:=[ ]+/=/'
test_eval_1="bar"
test_eval_2="baz"
```

--- @casey:
This seems reasonable. What's the specific use-case?

I think the right way to support this would be something like:

```just
just --evaluate --evaluate-format sh
```

It's long and inconvenient, but it allows supporting other formats in the future, e.g., other shells with different syntax.

Also, if need be in the future, we can figure out a more convenient alias, similar to `just --json` being a synonym for `just --dump --dump-format json`.

--- @jabalsad:
Thanks for looking at it!

> What's the specific use-case?

I have some complex justfile hierarchies, and in most of those applications, I will either have some useful environment variables encoded in the main justfile, or a dedicated file like `env.just` that only defines repo-wide or app-wide variables (used throughout the just recipes, or my shell environment within that context).

Having a single authoritative place where I can define them makes them easy to hook into other systems (like direnv, for example).

> It's long and inconvenient

Honestly, that bothers me the least; that's what shell aliases and things are for. A long but expressive syntax is good. And the point about supporting other formats make sense.

--- @casey:
Makes sense. And is the fact that they're exported or not in the justfile meaningful externally? I'm asking because I'm curious about whether it makes sense to prepend exported variables with `export`.

--- @casey:
Implemented in #3221. It hasn't made it into a release so it can still be tweaked if need be.

--- @casey:
The only thing I'm concerned about with this feature is the `export` keywords. That works in shells, but I'm not sure about `.env` implementations, I think that many `.env` implementations accept `export`, but I'm not sure all do.

--- @jabalsad:
I think, for that case, probably `--evaluate-format env` in the future makes sense. For my use-case, I generally need it for `sh` format instead so the `export` keyword is desirable.

--- @casey:
@jabalsad Makes sense, and `--evaluate-format env` is a good name.
