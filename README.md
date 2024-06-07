# Dreamluau

Dreamluau is a library for integrating [luau](luau.com) scripts into [BYOND](byond.com), and a successor to [auxlua](github.com/tgstation/auxlua). Scripts run in Dreamluau can read or modify BYOND values such as datums and lists, as well as call global or object-level BYOND procs.

## Usage

- Place `dreamluau.dll` (`libdreamluau.so` on Linux) in the same directory as your project's .dmb file.
- Place `api.dm` in your project and include it, preferably by ticking it to be included in your project's .dme file.
- Refer to the autodoc comments in `api.dm` for information on hook usage.
- Refer to [LuauDocumentation.md] for information on luau library interfacing with DM.

### Advanced Usage

The macros included in `api.dm` assume that the dreamluau binary is located in the current working directory, which in BYOND defaults to the directory of the .dmb file being run. If your project changes Dream Daemon's current working directory, or you do not wish to place the binary in the same directory as the .dmb, change the definition of `DREAMLUAU` in `api.dm` to point towards your intended location.

Dreamluau functions are called using the byondapi syntax of `call_ext`. To call the hook functions without using the macros, use the following expression, with `[function_name]` replaced with the name of the function you want to call:

```
call_ext([path to dreamluau], "byond:[function_name]")(...arguments)
```

## Requirements

Minimum supported BYOND version: 515.1631.
