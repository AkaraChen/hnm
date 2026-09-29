# ISSUE #2299 by @adsnaider: Allow defining private variables
Often, I need to define variables that are built from a smaller set of fundamental settings/variables. It would be useful to allow `[private]` to work on variables such that they can't be defined by the user of the script. Alternatively, the `_` prefix could also be used such that `just --variables` avoids listing them.

Happy to work on a PR if this is something that will be accepted

--- @casey:
Thanks for the report! Can you give an example of how a user redefining a variable would be a problem? I want to make sure  I understand the use-case.

I think allowing `[private]` on variables and, if present, not showing them in `just --variables`, would be pretty easy and reasonable.

--- @adsnaider:
Thanks @casey, I just finished migrating to `just` on my project so [here](https://github.com/adsnaider/Harmony/blob/main/justfile) you'll find examples where private variables would be nice to have (currently just using _ prefix)

--- @laniakea64:
The requested feature would also be useful to reduce clutter in `just --evaluate`.  That said, being able to view all variables' values is useful for debugging, so maybe a new additional flag (maybe `just --evaluate --debug` or maybe `just --evaluate --all`?) to also show evaluated private variables?

--- @adsnaider:
My implementation above specifically doesn't filter out private assignments on evaluation since that's useful information. I think you're right that it would be useful to filter them out by default and have an extra flag to flip that behavior. I'll see if I can implement that today
