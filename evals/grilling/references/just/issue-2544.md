# ISSUE #2544 by @casey: Guard sigil
We have `@` and `-` sigils which control linewise recipe execution. I thought that we could consider having a "guard" sigil which indicates that an error on a particular line should terminate the execution of the current recipe, but not the whole run.

For example, to make a recipe only execute if the environment variable `FOO` is set to `yes`:

```
foo:
  ?[[ $FOO == yes]]
  # the rest of the recipe
```

This is backwards incompatible, so would need a setting to opt-in to the new behavior. (Although it's unlikely to break most `justfile`, `?` at the beginning of the line is unlikely in `sh` and derivatives, and I think most scripting languages that people are likely to use.)

--- @liquidaty:
A further enhancement that would be nice is a way for the condition to be evaluated without invoking the shell e.g.:
```
foo:
  ?{{ assert(foo == 'yes') }}
  # the rest of the recipe
```

--- @casey:
@liquidaty There would have to be a different syntax for this, since I believe the pattern of having an interpolation first is common.

For example, if someone wanted to use Python to perform a test, they might do:

```
python := '/usr/bin/python3'

foo:
  ?{{ python }} …
```

--- @liquidaty:
> There would have to be a different syntax for this

Got it. Any syntax would be fine by me, but as I'm a new `just` user, that doesn't count for much.

That said, if it can be done, then, it seems logical (to me at least) to allow `{{ assert(...) }}` inside the recipe (and if it wasn't allowed before, it wouldn't seem to cause any compatibility issues to allow it). Currently, I can do this outside of a recipe:
```
a := if b == c { ... } else { ... }
d := assert(...)
```

and I can do this inside a recipe:
```
    {{ if b == c { ... } else { ... } }}
```

So it seemed logical to me that anything to the right of `:=` outside a recipe, could also be put inside `{{ }}` inside a recipe, and that therefore this would also work inside a recipe:
```
    {{ assert(...) }}
```

Though I guess maybe the issue is that assert raises an error or returns a different data type than a shell command's exit code.

--- @laniakea64:
> it seemed logical to me that anything to the right of `:=` outside a recipe, could also be put inside `{{ }}` inside a recipe

Correct.

> and that therefore this would also work inside a recipe:
> 
> ```
>     {{ assert(...) }}
> ```

Yes it does work.  I have justfiles that use `assert()` like that.

--- @anentropic:
> Yes it does work. I have justfiles that use `assert()` like that.

I was looking for something like an assert function...

Is it documented anywhere?  I didn't find it here https://just.systems/man/en/functions.html or via the search feature

--- @casey:
Added in #2547. I have one more feature I want to get in, but I'll hopefully cut a new release with it soon.

To make this change backwards compatible, I had to gate it behind a setting, `set guards`. When the `guards` setting is set, lines prefixed with `?` treat exit code `1` as skipping the rest of the recipe, but not failing the run or stopping execution of other recipes.

Error codes other than `0` and `1` are reserved and terminate execution with an error, so if needed we could give them some other meaning in a future version of Just, although just what that meaning might be I cannot dare to speculate.
