# ISSUE #2082 by @casey: Add `[cd(DIR)]` attribute
Allow setting the working directory of a recipe to an arbitrary directory. Good first issue!

--- @pedrofgodinho:
Pointing out a usecase for this: it'd facilitate cross-platform support. The project I'm working on currently will likely be developed on both windows and linux. `set windows-shell := ['powershell.exe']` is a good first step, but I couldn't find a better way to support both shells other than this hack:
```just
_cd_and_run dir *cmd:
    just _cd_and_run-{{os()}} {{dir}} {{cmd}}

_cd_and_run-linux dir *cmd:
    cd {{dir}} && {{cmd}}

_cd_and_run-macos dir *cmd:
    cd {{dir}} && {{cmd}}

_cd_and_run-windows dir *cmd:
    cd {{dir}}; {{cmd}}
```

Even this is fairly imperfect, as it doesn't easily allow you to do things like adding a `-` for ignoring errors (e.g. for a `lint` recipe to lint 2 subdirectories, when the linter gives exit code 1 when issues are found), among other similar problems. 

And adding things like `just _cd_and_run frontend npm run build` instead of just `npm run build` here and there isn't the prettiest solution...
An ideal solution would probably be:
```just
[cd(/frontend)]
build:
  npm run build
```

--- @artm:
I started working on this together with #2291. It works in simple cases: absolute path or relative path from current directory. I wonder though, where relative paths should actually be reconed from? Is there some precedent in just already which this feature should follow? What about modules?


--- @casey:
I think that relative paths should always be used relative to whatever would have been the working directory otherwise.

--- @artm:
Ok, I think that gives me enough direction, thanks.

--- @NoelJacob:
i think set working-directory globally and [cd(subdirectory/nested)] per command base would be good idea


--- @bcheidemann:
I've raised a [PR](https://github.com/casey/just/pull/2438) to implement this. However, I've implemented it as `[working-directory(DIR)]` instead of `[cd(DIR)]` since this is consistent with the `working-directory` setting.
