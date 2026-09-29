# ISSUE #2119 by @lisongmin: Variables defined in a submodule should be overridable on the command line

I have following justfile in the same directory:

var.just
```
compose_provider := 'podman'
```

test.just
```
import 'var.just'

show:
  @echo "compose_provider is {{ compose_provider }}"
```
Justfile
```
import 'var.just'
mod test 'test.just'

show:
  @echo "compose_provider is {{compose_provider}}"
```

And when i set the compose_provider from command line, the show recipe in the Justfile is changed as expect

```
❯ just --unstable compose_provider=nerdctl show
compose_provider is nerdctl
```

but the show recipe in the test module always show the default value

```
❯ just --unstable compose_provider=nerdctl test show
compose_provider is podman
```

The just version is 1.27.0

--- @casey:
This is currently not possible. On the CLI, only variables in the root justfile can be overridden, but it would be nice to be able to override submodule variables.

--- @lisongmin:
Thanks for the quick reply. And now I workaround by using the environment

```
compose_provider := env('COMPOSE_PROVIDER', 'podman')
```



--- @Scott-Guest:
@casey I'd be happy to implement this, but want to get clarity on the design first.

In the original example here, `compose_provider` at the top-level is actually a distinct variable from `compose_provider` in the submodule, so we probably want to require explicit qualification to disambiguate, e.g.
```
just test::compose_provider=nerdctl test show
```

That will get tedious with nested subcommands though, so maybe we could also allow the qualification after the subcommand like
```
just test compose_provider=nerdctl show
```
so that all of the following are equivalent:
- `just foo::bar::baz::key=value foo bar baz cmd` 
- `just foo bar::baz::key=value bar baz cmd`
- `just foo bar baz::key=value baz cmd`
- `just foo bar baz key=value cmd`

--- @kate-shine:
> we probably want to require explicit qualification to disambiguate

Do we though? Sure, if I call `just key=value foo::bar`, it's true that just currently sets that variable in the root Justfile, and not in `bar`, but it just does nothing (If I didn't miss anything). At that point, this might be an unnecessary added complexity that doesn't necessarily serve any purpose, and a change of behavior to set that variable in the module could be the way to go.

The proposed diambiguation might also be confusing when you alias those module recipes in root Justfile.

Another variant would be to combine explicit definition as you proposed, and deal with use cases where you want to set them in root module with the `Variables defined in one module can be referenced from another` issue from the https://github.com/casey/just/issues/2252

(You could just do `just key=value bar::baz` and bring `key` to `baz` with `super::key` or similar)



--- @casey:
Implemented in #3151. The full `::`-separated path to the variable must be used. I think that allowing whitespace-separated invocations, i.e. `just submodule variable=foo` would be a supreme nightmare. The invocation parser is already one of the most insane parts of the codebase, and this would make it much much worse 😅 Also, I think that allowing less than the full path would be ambiguous. For example, you would no longer be able to just override a variable `foo` in a root module if there was a submodule that also had a variable named `foo`.
