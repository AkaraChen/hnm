# ISSUE #1657 by @xavdid: Make the `else` branch of a conditional optional
I have the following recipe:

```just
@bump package:
    {{ if path_exists("Formula" / package + ".rb") == "false" { error("no formula for package " + package) } else { "" } }}
```

Which works great! But, it gives an error if I don't include a dud `else` block. 

```
error: Expected keyword `else` but found `'}}'`
  |
7 |     {{ if path_exists("Formula" / package + ".rb") == "false" { error("no formula for package " + package) } }}
```

It would be nice if it were optional

--- @casey:
All `just` expressions return a value, so we could only make this work if the `if` branch diverges, e.g., does not return. So it could work for the `error` case, but couldn't work in general. One possibility is to add an `error_if(CONDITION, MESSAGE)` function.

--- @Julian:
Why is that? I too was tempted into simply using `else { "" }` to workaround this (before finding this issue). Surely even if the if doesn't diverge it could simply default to an else branch with some default value like `""` as its "return value", which is highly likely to be ignored?

--- @mike-lloyd03:
I'm all for an `error_if` function. I'm currently using `else {""}` as a workaround but it's not very clean. I'm migrating most of my stuff over from `make` and enjoy the cleaner and simpler syntax of `just` so this solution seems like a good fit.

--- @casey:
I'm still not totally sold, but if we did implement this, how about a function called `ensure(CONDITION, MESSAGE)`, which takes a boolean CONDITION, and errors out with MESSAGE if it's false?

One reason I'm not totally sold is that it seems like this could be done in shell.

For example, for the use-case in the issue, you could do:

```
foo:
  test -e {{("Formula" / package + ".rb"}} || exit "no formula for package {{package}}"
```

--- @Julian:
> One reason I'm not totally sold is that it seems like this could be done in shell.

Personally I want to write literally the minimum amount of shell ever possible. I realize part of that is counter to using just at all, but certainly I try to minimize any shell I write regardless.

[Here](https://github.com/Julian/lean.nvim/blob/main/justfile#L30) are two places (the other a few lines down, where the condition is inverted) where I basically use the workaround discussed here -- I do have to say I personally find it surprising to be missing but I'm simply someone who's used / evaluated `just` once (to write that justfile) -- and certainly `ensure(...)` would be better than nothing so as a user I'd certainly prefer that if that's what you'd be comfortable with!

--- @xavdid:
+1 - I'm using `just`'s functions because I want to _avoid_ finicky shell scripts.

I think `ensure` would work here! sort of like an `assert` in other languages

--- @casey:
`assert` is probably a better name, since it's more familiar. Yah, agree about shell being tricky. And even if the `just` syntax is slightly worse, being statically typed is a big advantage.

Okay, I think I'm sold. If anyone wants to take a crack at this, it would make a good first PR!

--- @xavdid:
Nice, appreciate it!

I won't have time to look at this for a few months (and I have to brush up on my Rust). But if it's still available then, I'll take a crack at it!

--- @xavdid:
Awesome, thank you!
