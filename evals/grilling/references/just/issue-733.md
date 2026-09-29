# ISSUE #733 by @EdenEast: Option for .env variables to override existing
I have run into a use case were I would like the variables defined in a `.env` file to override
any already defined variables.

The case is in my nix dotfiles configuration. I would like the option to override the `$USER` env
variable when I build and install the configuration depending on the machine I am installing it on.

Having a setting in the justfile would be helpful

```
# Controls whether the variables defined in a dotenv file will override already defined variables
set dotenv-override := true
```

(name `dotenv-override` in similar ideas in #469)

--- @casey:
I think this would definitely be a welcome feature.

Another way this could be implemented, which would allow a little more granularity, is by adding a `dotenv(…)` function, so you could do:

```
export USER := dotenv('USER')
```

--- @EdenEast:
Having a function like `dotenv()` would be useful. I think it would work nicely with `env_var()`. Would my assumption be
correct about the order of priority:

`dotenv` would use the value defined in `.env` first and then the user's env while `env_var` would use the users env
first then check the `.env` file.

If so that would be an easy way for the user to decide what should take priority.


--- @casey:
Yup, that's right.

--- @EdenEast:
I started to look into the code to see how to implement this and see that if this change was made it would break the current behaviour of `env_var`. Currently `env_var` checks `.env` first then `std::env`. Current function for reference:

```rust
fn env_var(context: &FunctionContext, key: &str) -> Result<String, String> {
  use std::env::VarError::*;

  if let Some(value) = context.dotenv.get(key) {
    return Ok(value.clone());
  }

  match env::var(key) {
    Err(NotPresent) => Err(format!("environment variable `{}` not present", key)),
    Err(NotUnicode(os_string)) => Err(format!(
      "environment variable `{}` not unicode: {:?}",
      key, os_string
    )),
    Ok(value) => Ok(value),
  }
}
```

--- @casey:
Ah, okay. I guess then you could use `env_var` to do what you want to do:

```
export USER := env_var('USER') # will check .env first
```

--- @EdenEast:
I created a test to see if this would work. It looks like this does not work. Might have something to do with the order of `env_var` and `load_dotenv.rs`.

```sh
➜ echo $USER
eden

➜ cat .env
USER=other

➜ cat justfile
export USER := env_var('USER')
check:
  @echo $USER

➜ just check
eden
```

--- @casey:
Ah yeah, that's right. If you look at `load_dotenv`, it doesn't load variables that are already in the environment: 

```rust
use crate::common::*;

pub(crate) fn load_dotenv(
  working_directory: &Path,
) -> RunResult<'static, BTreeMap<String, String>> {
  // `dotenv::from_path_iter` should eventually be un-deprecated, see:
  // https://github.com/dotenv-rs/dotenv/issues/13
  #![allow(deprecated)]
  for directory in working_directory.ancestors() {
    let path = directory.join(".env");

    if path.is_file() {
      let iter = dotenv::from_path_iter(&path)?;
      let mut dotenv = BTreeMap::new();
      for result in iter {
        let (key, value) = result?;
        if env::var_os(&key).is_none() {
          dotenv.insert(key, value);
        }
      }
      return Ok(dotenv);
    }
  }

  Ok(BTreeMap::new())
}
```

--- @kotx:
Something that could be related is if `just` loaded `.env.*` files, like `.env.production` or `.env.local` or `.env.production.local` files, which overwrite the existing .env configuration:
https://github.com/bkeepers/dotenv#what-other-env-files-can-i-use

--- @optimuspaul:
> load_dotenv, it doesn't load variables that are already in the environment:

Is this actually desired behavior? I use .env specifically to override some default variables. My workaround is to `source .env` in almost every one of my recipes.

--- @runeimp:
As I recall the general consensus in such things is this order of importance

1. Command line options
2. Environment Variables
3. Config file
4. `.env` type file support

The thinking being that `.env` files and similar systems are often stored in a repo and act as reasonable defaults for developers that don't want to pollute their local environment with projects specific settings that may change often and should be consistent for a given release version or branch.

Config files may, but rarely do, exist in a repo except as an example config which doesn't actually get used. This can change depending on the needs of the application as a default config file might need to be more important than variables already defined in the environment. But this is not common.

System environment variables that are project specific should have high priority as the are, generally, only expected to be set in testing or production environment systems.

Command line options that match against config or env vars have the highest priority as they are expected to only be used in development or for very specific situations. Such as when a config values or env vars are expected to be the cause of problems or for A/B testing.

--- @optimuspaul:
Right, of course that makes sense now. it does seem to be a common thing people ask for, but seems like an edge case.

--- @runeimp:
What seems like an edge case, the need for such a setup, or the need to override local system variables with those stored in a file or else where? In actuality both are edge cases depending on the type of projects you work on.

The four layered approach I mention is very common in server development, especially when it involves deployment to the cloud or intranet cloud services where env vars are the main form of node specific configuration (instance name, IP address, etc.) and YAML or JSON are the main form of config file.

Many languages have facilities to setup a project environment using unique env vars. If you're programming in Python you can use the built-in [venv](https://docs.python.org/3/library/venv.html) or [Pipenv](https://pipenv.pypa.io/) module to setup a virtual environment. I believe Node allows for setting up variables per project. Plus there are many tools like as [direnv](https://direnv.net/) which is commonly used for such things, and [LocalStatus](https://github.com/tylermumford/localstatus) which looks very promising, etc. And of course Just will read a `.env` if you have <code>[set dotenv-load](https://just.systems/man/en/chapter_26.html)</code> in your Justfile.

If you expand in even a fraction of what all that programming can cover edge cases high everywhere. There is no ESCAPE! AHAHAHAHAHAHAHAH hahddhash... ha? Yeah, something like that. 😇

--- @optimuspaul:
😂 umm yeah, I was referring to overriding already set variables being an edge case. I’m not clear on what the other “such a setup” is, but I don’t need that explanation. I already use the 4 layer dip approach except I don’t like tomatoes or guacamole so I don’t use config files, I also only use .env for local development. Those are never deployed for my projects. Don’t forget about the corner cases 🤓
