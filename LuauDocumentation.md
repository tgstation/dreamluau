# Objects

Datums, lists, typepaths, static appearances, and some other objects are represented in Luau as userdata. Certain operations can be performed on these types of objects.

## Common metamethods

The following metamethods are defined for all objects.

### \_\_tostring(): string

Returns the string representation of the object. This uses BYOND's internal string conversion function.

### \_\_eq(other: any): boolean

Compare the equality of two objects. While passing the same object into luau twice will return two references to the same userdata, some DM projects may override the equality operator using an `__operator==` proc definition.

## Datum-like Objects

Datum-like objects include datums themselves, clients (if they have not been redefined to be children of `/datum`), static appearances, and the world.

### \_\_index(index: string): any

Access the member specified by `index`.

If `index` is a valid var for the object, the index operation will return that var's value.
If the var getting wrapper proc is set, the operation will instead call that proc with the arguments `(object, index)`.

For objects other than static appearances, if `index` is a valid proc for the object, the operation will return a wrapper for that proc that can be invoked using call syntax (e.g. `object:proc(...arguments)`). If the object proc calling wrapper is set, calling the returned function will instead call the wrapper proc with the arguments `(object, proc, {...arguments})`. Note that vars will be shadowed by procs with the same name. To work around this, use the `dm.get_var` function.

### \_\_newindex(index: string, value: any): ()

Set the var specified by `index` to `value`, if that var exists on the object.

If the var setting wrapper proc is set, the operation will instead call that proc with the arguments `(object, index, value)`.

## Lists

Lists are syntactically similar to tables, with one crucial difference.
Unlike tables, numeric indices must be non-zero integers within the bounds of the list.

### \_\_index(index: any): any

Read the list at `index`. This works both for numeric indices and assoc keys.
Vars lists cannot be directly read this way if the var getting wrapper proc is set.

### \_\_newindex(index: any, value: any): any

Write `value` to the list at `index`. This works both for writing numeric indices and assoc keys.
Vars lists cannot be directly written this way if the var setting wrapper proc is set.

### \_\_len(): integer

Returns the length of the list, similarly to the `length` builtin in DM.

### Iteration

Lists support Luau's generalized iteration. Iteration this way returns pairs of numeric indices and list values.
For example, the statement `for _, v in L do` is logically equivalent to the DM statement `for(var/v in L)`.

# Global Fields and Modules

In addition to the full extent of Luau's standard library modules, some extra functions and modules have been added.

## Global-Level Fields

### sleep(): ()

Yields the active thread, without worrying about passing data into or out of the state.

Threads yielded this way are placed at the end of a queue. Call the `awaken` hook function from DM to execute the thread at the front of the queue.

### loadstring(code: string): function

Luau does not inherently include the `loadstring` function common to a number of other versions of lua. This is an effective reimplementation of `loadstring`.

### print(...any): ()

Calls the print wrapper with the passed in arguments.
Raises an error if no print wrapper is set, as that means there is nothing to print with.

### \_state_id: integer

The handle to the underlying luau state in the dreamluau binary.

## \_exec

The `_exec` module includes volatile fields related to the current execution context.

### \_next_yield_index: integer

When yielding a thread with `coroutine.yield`, it will be inserted into an internal table at the first open integer index.
This field corresponds to that first open integer index.

### \_limit: integer?

If set, the execution limit, rounded to the nearest millisecond.

### \_time: integer

The length of successive time luau code has been executed, including recursive calls to DM and back into luau, rounded to the nearest millisecond.

## dm

The `dm` module includes fields and functions for basic interaction with DM.

### world: userdata

A static reference to the DM `world`.

### global_vars: userdata

A static reference that functions like the DM keyword `global`. This can be indexed to read/write global vars.

### global_procs: table

A table that can be indexed by string for functions that wrap global procs.

Due to BYOND limitations, attempting to index an invalid proc returns a function logically equivalent to a no-op.

### get_var(object: userdata, var: string): function

Reads the var `var` on `object`. This function can be used to get vars that are shadowed by procs declared with the same name.

### new(path: string, ...any): userdata

Creates an instance of the object specified by `path`, with `...` as its arguments.
If the "new" wrapper is set, that proc will be called instead, with the arguments `(path, {...})`.

### is_valid_ref(ref: any): boolean

Returns true if the value passed in corresponds to a valid reference-counted DM object.

### usr: userdata?

Corresponds to the DM var `usr`.

## list

The `list` module contains wrappers for the builtin list procs, along with several other utility functions for working with lists.

### add(list: userdata, ...any): ()

Logically equivalent to the DM statement `list.Add(...)`.

### copy(list: userdata, start?: integer, end?: integer): userdata

Logically equivalent to the DM statement `list.Copy(start, end)`.

### cut(list: userdata, start?: integer, end?: integer): userdata

Logically equivalent to the DM statement `list.Cut(start, end)`.

### find(list: userdata, item: any, start?: integer, end?: integer): integer

Logically equivalent to the DM statement `list.Find(item, start, end)`.

### insert(list: userdata, index: integer, ...any): integer

Logically equivalent to the DM statement `list.Insert(item, ...)`.

### join(list: userdata, glue: string, start?: integer, end?: integer): string

Logically equivalent to the statement `list.Join(glue, start, end)`.

### remove(list: userdata, ...any): integer

Logically equivalent to the DM statement `list.Remove(...)`.

### remove_all(list: userdata, ...any): integer

Logically equivalent to the DM statement `list.RemoveAll(...)`.

### splice(list: userdata, start?: integer, end?: integer, ...any): ()

Logically equivalent to the DM statement `list.Splice(start, end, ...)`.

### swap(list: userdata, index_1: integer, index_2: integer): ()

Logically equivalent to the DM statement `list.Swap(index_1, index_2)`.

### to_table(list: userdata, deep?: boolean): table

Creates a table that is a copy of `list`. If `deep` is true, `to_table` will be called on any lists inside that list.

### from_table(table: table): userdata

Creates a list that is a copy of `table`. This is not strictly necessary, as tables are automatically converted to lists when passed back into DM, using the same internal logic as `from_table`.

### filter(list: userdata, path: string): userdata

Returns a copy of `list`, containing only elements that are objects descended from `path`.

## pointer

The `pointer` module contains utility functions for interacting with pointers.
Keep in mind that passing DM pointers into luau and manipulating them in this way can bypass wrapper procs.

### read(pointer: userdata): any

Gets the underlying data the pointer references.

### write(pointer: userdata, value: any): ()

Writes `value` to the underlying data the pointer references.

### unwrap(possible_pointer: any): any

If `possible_pointer` is a pointer, reads it. Otherwise, it is returned as-is.
