# ISSUE #2872 by @MadBomber: import statement appends recipes to end of main justfile ??
I'm just getting around to converting all of just files from the `justprep` era to use the native import and mod commands.  I'm surprised to see that even though I have a import statement at the topp of my justfile, that the recipes from the import and not placed into the location whe the import statement resides.  it appears that those imported recipes are appended to the end of the main justfile.  this is contrary to common sense.

tasks coming from a import statment at the top of a justfile should be PRE-pended not A-pended to the list of recipes.  For example I have a list taks that is customized for the way that I want stuff to look in all my just environments.  It also has settings that I want all of my main justfiles to have.

In my home directory I have a justfile named ".justfile"  In all of justfiles for all projects I have at the top of my justfiles "import '~/.justfile'"


--- @casey:
What do you mean when you say that the imported recipes are appended to the end of the main justfile? Do you mean the order they appear in `--list`, or something else?

--- @MadBomber:
Hi @casey .  You've made lots of good progress on just and its become a common work tool for may people.

for me the file ~/.justfile contains lots of setting and a set of common tasks that I want every justfile that I use for my different project to have available.  The first task that the ~/.justfile has is a `list` recipe which defines how I want my stuff to be shown.  Its consistent on every project.

However, then I do a project file that looks like this:

```
import '~/.justfile'
xyzzy:
  echo "its magic"
```

The `xyzzy` task becomes the default task and not the first task `list` defined in the global ~/.justfile

in my project when I do
```
just
```

I will get back "its magic" instead of the expected list of tasks that were defined in the imported file.

--- @casey:
Gotcha. Which recipe is the default in a justfile with imports is tricky. I think different people will prefer different behavior, i.e. first recipe in top-level justfile is the default, versus first recipe in first import is the default.

Probably the way to solve this is to add a `[default]` attribute, which may appear at most in a module, including in imported recipes, and which overrides the usual order-based default recipe. So if you want a default recipe from an import, you can give it the `[default]` attribute, and not worry about order.

--- @casey:
I added a `[default]` attribute in #2878, which can be used to override the usual behavior.

--- @MadBomber:
I'm not sure that having the default in the imported file works.  I had to add the default to the main justfile after all of my imports.    Your task running is still my goto.  I use it everywhere.

--- @casey:
I'm not sure I understand. If you want the default recipe to be in the import, you can add `[default]` to it. If you don't, you you either add `[default]` in the top-level justfile, or you can rely on the existing behavior.

--- @laniakea64:
> I added a `[default]` attribute in [#2878](https://github.com/casey/just/pull/2878), which can be used to override the usual behavior.

Should an **explicitly-specified** `[default]` in a root justfile (or top-level justfile of a module) be allowed to override any imported `[default]`s?

For example:

`justfile`
```
import 'a.just'

[default]
foo:
  echo 'top-level default'
```

`a.just`
```
[default]
a:
  echo 'imported default'
```

This is currently an error, but since `justfile` is the top-level, should its explicitly-specified `[default]` be allowed and override any `[default]` recipes from `import`ed justfiles?

In addition to being intuitive, having this option would be useful for an imported justfile that is shared between several justfiles and contains a recipe that should be default in most but not all cases where it's imported.

--- @casey:
> Should an **explicitly-specified** `[default]` in a root justfile (or top-level justfile of a module) be allowed to override any imported `[default]`s?

Hmmm, I can see the motivation. I'm not sure though. In general, I don't like features which behave differently depending on whether they're in an import or not.

Fortunately since it's currently an error it could be eventually allowed without breaking backwards compatibility. Feel free to create an issue, and if it gets enough support I could definitely see adding it.
