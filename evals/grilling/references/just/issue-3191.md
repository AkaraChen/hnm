# ISSUE #3191 by @casey: Add setting to make `--list` the default action
This is apropos of #2845, which added `--justfile` and `{{just_executable()}}` to the suggested default list recipe in the readme.

Since it's common to have the default recipe list recipes, I think the best approach is actually to add a setting which enables this directly, instead of having to define a recipe at all:

```just
set default := list
```

This avoids recursively invoking `just` and needing to forward arguments to the recursive invocation.

--- @woutervh:
I'm currently using:

```
# Display all recipes
[default]
_list:
    @ {{just_executable()}} --list --justfile {{justfile()}} --unsorted
```


The performance difference is huge (just 1.48.0) :
```
$ Measure-Command { just }
Seconds           : 6
Milliseconds      : 809
Ticks             : 68093988
TotalSeconds      : 6,8093988
TotalMilliseconds : 6809,3988
```


````
$ Measure-Command { just --list }  
...
Seconds           : 0
Milliseconds      : 27
Ticks             : 271894
TotalSeconds      : 0,0271894
TotalMilliseconds : 27,1894
```

--- @jamescooke:
@woutervh Thanks for pointing out the use of `--justfile {{justfile()}}` ... I've just run into the scenario where I invoked `just -f other_justfile`, which had the default as simply `@just --list`. This listed the recipes from the _default_ justfile and not from `other_justfile` and I was super confused 😵‍💫 

--- @casey:
I started implementing this, but I'm actually not sure if this should be a setting, as in `set default := list`, or a flag `--default list`. Flags can be set via environment variable, which would mean you could do `JUST_DEFAULT=list`, and have it work for all justfiles, without needing to add a setting.

Both options are easy, so we could do both, but if this is really about individual user preference (i.e., users either prefer a default recipe that does something or to list recipes, in all cases) as opposed to being a per-justfile preference (i.e., in some justfiles users want a default recipe and in some they want to list recipes), then the flag/environment variable makes more sense.

--- @hellmrf:
@casey, I think `set default := list` is the more consistent choice, since it applies uniformly to everyone working on the codebase and makes the documentation easier to follow.

In my projects using Just, I typically include a `README.md` with basic instructions, and it's much simpler to tell new developers to run `just` to list the available recipes in the base module (or `just <submodule>` to see the recipes within a specific module).

I'd even go a bit further and argue that `set default := list` should be the default configuration. In my experience, it's relatively uncommon to have a single command important enough to justify being run with just `just`. I also tend to run `just list` whenever I switch repositories (even ones I actively work on), since it takes almost no time and helps avoid confusion. In practice, I end up manually implementing @woutervh's approach in nearly all of my projects.


--- @casey:
I added a setting, `set default-list` which can be used for an individual justfile, as well as a flag, `--default-list`, and an environment variable `JUST_DEFAULT_LIST`, if you want to default to listing recipes globally.
