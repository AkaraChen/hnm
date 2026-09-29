# ISSUE #2291 by @piotrekkr: [Feature Request] An option to disable change directory for all recipes in justfile
Hello. `just` is a great tool and I really like it. However this automatic change directory behavior and using `[no-cd]` in all my recipes is a bit annoying (but still acceptable). 
Would you consider adding some setting to disable this behaviour? Something like `set no-cd` and new attribute `[do-cd]` or something like this? 

### My use case. 

All projects I'm working on are in `~/Projects`. 
```
~/Projects/my-project-1
~/Projects/my-project-2
...
```
I want to reuse one `justfile` located in `~/Projects/justfile`. All those projects contain terraform configuration that I'm working on. When I want to change infrastructure of some project I will:
1. Go into `~/Projects/my-project-2` 
2. Make changes in infra config
3. Run `validate`, `fmt`, `plan` 
4. If all is okay then `apply` infrastructure

There are some other common actions I do on all projects also but this is not important. 

To work with `terraform` I need to be inside the directory that contains configuration. To make it work with `just` each recipe needs to have this `[no-cd]` added.

My working `justfile` looks like similar to this:
```
set quiet := true

[private]
[positional-arguments]
run-tf *args='':
    # this is actually more complicated command using docker with volumes and whatnot
    # but for this example I simplified it
    terraform "$@"

[no-cd]
init:
    just run-tf init

[no-cd]
upgrade:
    just run-tf init -upgrade

[no-cd]
validate:
    just run-tf validate

[no-cd]
plan:
    just run-tf plan

[no-cd]
apply:
    just run-tf apply

[no-cd]
destroy:
    just run-tf destroy

[no-cd]
cli:
    just run-tf bash

[no-cd]
format:
    just run-tf fmt -recursive .

[no-cd]
output *args='':
    just run-tf output "$@"

alias console := cli
```

All is working fine but would be great if I could just use something like `set no-cd` and remove all those `[no-cd]` lines. If I wanted to change dir to jsutfile dir I could then add `[do-cd]` in recipe and it would work the standard way.

Would you consider adding setting like this? 

Thanks

--- @casey:
I think `set no-cd` would be reasonable. The working directory logic has been recently refactored into a single function, `ExecutionContext::working_directory`, which has access to settings, so it should actually be a pretty simple PR.

--- @artm:
I have started tackling this. I think it can be combined with #2082, to avoid multiplying attributes beyond necessity:

- `set no-cd` makes that by default working directory isn't modified
- `[cd(DIR)]` sets alternative working directory for a recipe with or without global `no-cd` setting
- `[cd]` undoes global `set no-cd` for a marked recipe (instead of the suggested `[do-cd]`
- setting `[cd]` and `[no-cd]` on the same recipe should cause an error on startup saying they are incompatible.




--- @casey:
@artm That all seems reasonable to me!

--- @rnhurt:
I just started using `just` and I'm really loving it.  However, I'm running into the same problem as the OP.  I have a bunch of AWS CloudFormation commands that are dependent on the directory they execute in.  

Currently, my configuration works but is littered with `[no-cd]` attributes.  This change would clean up my "justfile" considerably.

--- @waynr:
Adding my two cents here. I have a global justfile that I expect to be able to call from anywhere on my filesystem, eg `cd /home/myhome/myprojects/myproject-git-root/subdirectory && just -g my-global-justfile-target`

My expectation when doing this is that the working directory for the just process would be `/home/myhome/myprojects/myproject-git-root/subdirectory`. But for some reason it ends up being `/home/myhome/myprojects/myproject-git-root`, unless I rename the `.git` directory in `myproject-git-root`.

Why would anyone expect the behavior here to be implicitly using the git root as the working directory for the just targets? Should I file this as a separate bug issue?
