# ISSUE #2908 by @orlp: Configurable JUSTFILE_NAMES?
I'd like to enable the workflow where I have a `.localjustfile` in project directories to store commands for me personally, not intended to be committed to the repository. Simply using `.justfile` is not sufficient if the project itself uses `just`, as `just` will give an error if both `.justfile` and `justfile` exist, stating multiple candidates have been found (which makes sense).

Just has the `-f, --justfile` argument to specify the exact justfile to load, but this does not do the automatic "scan upwards searching for `<filename>`" behavior of `just` that nicely lets you run commands from anywhere in the project.

Would you be open to add an argument similar to `--justfile`, but which instead changes the name `just` searches for, rather than specifying an exact path?

Or alternatively (but less flexibly) an option `--local-justfile` which makes `just` look for `localjustfile` or `.localjustfile`.

--- @woutervh:
You can do an optional import in the main justfile:

```
import? '.localljustfile'
```


Personally I would like to avoid hardcoded filenames, and support all *.justfile in  one or more directories via wildcard-support
see https://github.com/casey/just/issues/1885

```
import? '.just/*.justfile'
import? '~/.just/*.justfile'

```

--- @orlp:
@woutervh That still requires a 'main justfile' to exist and accommodate this workflow. The goal is to have a workflow which is entirely local and does not depend on or influence the project it lives in in any way.

--- @woutervh:
have you seen this:
https://just.systems/man/en/global-and-user-justfiles.html

```
just --global-justfile
```

--- @orlp:
Yes, I don't want a global justfile, I want something local (so not committed into git), on a per-project basis.

--- @casey:
Seems reasonable to me! Implemented in #3234 with a new `--justfile-name` option, which can be used to supply any number of justfile names, which are checked in order. It can also be set with the `JUST_JUSTFILE_NAME` environment variable. If `--justfile-name` is used with `--init`, it sets the name of the justfile that's created to the first argument.

--- @orlp:
@casey Thanks, I think that does exactly what I want, I will check it out.
