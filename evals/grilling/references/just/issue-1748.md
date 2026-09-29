# ISSUE #1748 by @evandam: Allow loading multiple dotenv files
Related: https://github.com/casey/just/issues/945#issuecomment-1828806192

It would be great to be able to source multiple dotenv files in a "cascading" manner, similar to https://github.com/bkeepers/dotenv#what-other-env-files-can-i-use. This allows per-environment overrides, and a `.env` to provide defaults to fall back on (meaning `.env.test`, `.env.qa`, `.env.prod` don't need to repeat the same common env vars that can be defined once in `.env`).

I think there may be some related work that would need to be done to be able to interpolate filenames in `dotenv-files`, though.

```
ENV := "prod"
set dotenv-files := [".env.{{ENV}}", ".env"]
```

And call with `just ENV=prod` to load env vars from `.env.prod`, and then `.env` (assuming dotenv files don't overwrite existing environment variables).

As far as behavior goes, it probably makes sense to fail silently (or log it) if a file in the array isn't found since they can just be "candidates" for dotenv files to load. For example, `.env.local` may not always be present in an environment, but we'll try to load it if it does happen to exist.

--- @azriel49:
it would be super useful to be able to override the default values ​​of the .env using a .env.local !!
Like the OP, i usually put my .env into the repository with default values, and override some confs with an environment specific .env.local

--- @r3nor:
This would be great, many project use multiple dotenvs and I have to use bash scripts every time

--- @sbeckeriv:
how about allowing more then on dotenv-filename config? 

`just --dotenv-filename=.env --dotenv-filename=.env.local command`

and 

```
      ENV := "prod"
      set dotenv-load
      set dotenv-filename := ".env"
      set dotenv-filename := ".env.local"
      set dotenv-filename := ".env.{{ENV}}"
      set dotenv-path := "path"
```

if they can be evaluated in order, last one wins. I was reading the code and I think updating the config to allow an vec of filenames and then modifying the current dotenv load process might be a smaller change. The code path would always be used making it less likely to have diverging behaviors. one could even skip supporting the command line part and make it a set config only. 

[edit]
there are a number of things that would need to be cleaned up, however, this is a diff of it working with arguments and settings. I would want to know if this was an acceptable way forward before spending more time on it. 

https://gist.github.com/sbeckeriv/664c3dd8a38c84c877a45f50d44a0f20 

While making this diff I learned that `[]` syntax is not supported currently. I believe there is other issues around supporting 'arrays' of values.

--- @nk9:
If we're going to allow multiple dotenv files, it seems like requiring them all to be in a single `dotenv-path` is an unnecessary constraint. I would agree with OP that we should have a new setting called `dotenv-files` which would take an array of paths including the filename, e.g.:

```
set dotenv-files := [".env", "src/dir1/.env2"]
```

Question, though: how should this setting work with fallback justfiles?

--- @mikeyhew:
I'm interested in getting this feature added since it would be pretty helpful for my workflow. I saw that there is an open PR from last year (#2022) that implements this by adding a new setting, dotenv-files, which is an array of strings and conflicts with dotenv-path and dotenv-filename. There was feedback that it could be combined with dotenv-filename and have it take an array in order to avoid adding a third conflicting setting.

I'm considering working on a new PR either from scratch or building off of #2022, but I'm wondering what the best solution is in terms of settings.

I'm assuming our goals are:
- don't break any existing Justfiles
- make it possible to load from multiple .env files and have the later ones override the earlier ones
- if possible, don't add a third setting since we already have 2
- think about fallback justfiles and how this interacts with that
- ideally, "finish" this feature so that it handles all the reasonable use cases we can think of, and we don't need to add another setting in the future to support another use case. e.g. decide now if we want to support glob patterns, or interaction with variables like ENV

Here's the current state and a proposed implementation that modifies the existing settings by letting you provide an array.

## Current State

The current state of things is, we have both the dotenv-filename and dotenv-path settings. They conflict with each other so you can only use one or the other (or none).

The way dotenv-filename works is, you supply a filename (although it looks like it doesn't have to be a filename like `foo.env`, it could also be a relative path like `config/foo.env`), and then it joins that with the working directory and checks for a file at that path, and if there is one it loads it. If not then it does the same with the parent of the working directory, then the parent of that, and so on.

The way dotenv-path works is you supply a relative or absolute path and it loads the file at that path and then you're done.

## Modifying the existing settings

Let's think about how we would modify each of these settings to support multiple .env files:

For dotenv-path, we modify the setting to optionally take an array. If it is an array we load each of the files in the array in succession. If any files are not present we ignore them just like we do when only one file is specified.

For dotenv-filename, we'd also modify the setting to optionally take an array. The current code calls `working_directory.ancestors()` and loops through each ancestor until it finds a file at `path.join(filename)`, then loads that file. Here's the code:

```rust
for directory in working_directory.ancestors() {
  let path = directory.join(filename);
  if path.is_file() {
    return load_from_file(&path);
  }
}
```

We would modify that to check every filename in the array, and if any of those files is present, we load the files that are present in the order they are specified in the array. Otherwise we continue. Like this:

```rust
for directory in working_directory.ancestors() {
  let paths_with_files_present = filenames.iter().filter_map(|filename| {
    let path = directory.join(filename);
    if path.is_file() {
      Some(path)
    } else {
      None
    }
  }).collect::<Vec<_>>();

  if !paths_with_files_present.is_empty() {
    return load_from_files(&paths_with_files_present);
  }
}
```

## Referencing variables in the filename or path

In the issue description, @evandam requested that this feature support referencing variables, for example:

```
ENV := "prod"
set dotenv-files := [".env.{{ENV}}", ".env"]
```

I can see how this would be useful and in theory it could be supported by interpolating variables in the string when loading the dotenv files. But according to [this comment](https://github.com/casey/just/issues/2022#issuecomment-2121209850_), it sounds like dotenv files are loaded too early to reference variables from the Justfile, however there's a new kind of string literal that might make it possible? I'm not clear on what @casey is saying here:

> It will have to take a static string. dotenv file settings are applied before anything in the justfile is evaluated, so it can't use variables. I did merge #2055 recently, which adds a new kind of string literal which can contain environment variables, so those can be used.

In any event, this could be left out of the initial implementation as long as it is possible to add it in the future without breaking backward compatibility. As long as we don't worry about breaking things for people who use weird filenames that include the literal characters `{{ENV}}` in the actual file name, I think we're fine.

## Interaction with fallback Justfiles

@nk9 asked how this setting would interact with fallback justfiles. The answer is pretty straighforward: the same way it does today. If a command isn't present then Just will try again using whatever justfile it finds in a parent or ancestor directory. There's no interaction between the settings in a Justfile and the settings in a fallback Justfile.

The main usecase I can think of would work, just like it works today: if root/Justfile and root/child/Justfile are both present and you have a `.env` file in root but not in root/child, then when root/child/Justfile runs it will load root/.env because there's not root/child/.env. If you specified `dotenv-filename = [".env", ".env.local"] and `root/.env` and `root/.env.local` are both present but `root/child/.env` and `root/child/.env.local` are not present, then when `root/child/Justfile` runs it will load both `root/.env` and `root/.env.local`

--- @mikeyhew:
I opened a PR that I think addresses everything in this issue: https://github.com/casey/just/pull/2682

For example if you want to load `.env`, then `.env.$ENV`, then `.env.local`, you can do:

```just
set dotenv-filename = [".env", x'.env.${ENV:-dev}', ".env.local"]
```

--- @bionicles:
hi, i could use this to facilitate independent storage of configuration and secrets, please consider merging #2682 

--- @lordsoffallen:
This is very much needed!

--- @mikeyhew:
@casey I'd be happy to fix the merge conflicts and lint failures on my PR #2682 if you confirm that you want it and you'll have time to review it - let me know

--- @k0pernikus:
I happen to use [mise-en-place](https://mise.jdx.dev) with which I also install `just`

```
mise use just --pin
```


And in its config `.mise.toml` I can load multiple env:

```toml
[tools]
just = "1.50.0"

[env]
_.file = [".env", ".env.local"]
```

Last file defined takes precedence, offering me an `.env` with defaults and an `.env.local` for `.gitignored` secrets.

The ENV variables are defined after running mise install for the current shell. (Though mise itself also provides a tasks to setup and to be run via `mise run`; so one may also replace just entirely, though I prefer the just experience.)

--- @casey:
Finally implemented in #3412, gated on `set lists`. See https://github.com/casey/just#lists and #3377.

--- @dajoha:
Thank you so much to have implemented it @casey , thank you so much @mikeyhew for initial PR; it was the very blocking point to add a Justfile to the project I'm working on ❤️
