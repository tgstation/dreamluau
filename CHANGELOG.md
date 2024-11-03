#Changelog

## [0.1.4]

### Additions

- Individual states can have an execution limit override set with `set_state_execution_limit_millis` and `set_state_execution_limit_secs`, and cleared with `clear_state_execution_limit`. A state's execution limit override takes precedence over the global execution limit.

## [0.1.3]

### Additions

- An optional argument `isolated` can now be passed to `DREAMLUAU_NEW_STATE`. The following changes are applied to a state initialized with a truthy value of `isolated`:
  - The `dm`, `list`, and `pointer` modules are not added to the global environment
  - Function call and thread resume arguments other than `null`, numbers, and strings, are converted to `nil`

## [0.1.2]

### Changes

- A more informative error message has been provided when attempting to convert destructed userdata to a BYOND value.

## [0.1.1]

### Fixes

- Bumps the dependency for `time`, fixing certain failures being experienced by users building it from source in their own CI workflows.

## [0.1.0]

- Initial Release
