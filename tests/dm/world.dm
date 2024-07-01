#include "../../dmsrc/api.dm"

/proc/print(_state_id, list/arguments)
	world.log << arguments.Join(" ")

/world/Topic(T, address)
	var/list/params = params2list(T)
	var/test_to_run = params["test"]
	if(test_to_run)
		try
			call("/proc/[test_to_run]")()
		catch(var/exception/e)
			return "[e]"

#define TEST(name, body, cleanup...) /proc/##name() \
{ \
	var/state = DREAMLUAU_NEW_STATE(); \
	var/error; \
	try \
	{ \
		##body \
	} \
	catch(var/e) \
	{ \
		error = e; \
	} \
	##cleanup \
	DREAMLUAU_KILL_STATE(state);\
	if(error) \
	{ \
		throw e; \
	} \
}

TEST(hello_world,
	DREAMLUAU_SET_PRINT_WRAPPER("/proc/print");\
	var/result = DREAMLUAU_LOAD(state, "print(\"Hello World!\")");\
	assert_result(result, "finished", 0),
	DREAMLUAU_SET_PRINT_WRAPPER(null);)

TEST(usr_pushing,
	var/mob/M = new();\
	DREAMLUAU_CALL(set_usr)(M);\
	var/result_1 = DREAMLUAU_LOAD(state, "return dm.usr");\
	assert_result(result_1, "finished", list(M));\
	var/result_2 = DREAMLUAU_LOAD(state, "return dm.usr");\
	assert_result(result_2, "finished", list(null)))

TEST(calling,
	var/result_1 = DREAMLUAU_LOAD(state, "function foo() return \"foo\" end");\
	assert_result(result_1, "finished", 0);\
	var/result_2 = DREAMLUAU_CALL_FUNCTION(state, list("foo"), list());\
	assert_result(result_2, "finished", list("foo")))

TEST(sleeping,
	var/result_1 = DREAMLUAU_LOAD(state, "sleep() return \"foo\"");\
	assert_result(result_1, "sleep");\
	var/result_2 = DREAMLUAU_LOAD(state, "sleep() return \"bar\"");\
	assert_result(result_2, "sleep");\
	var/result_3 = DREAMLUAU_AWAKEN(state);\
	assert_result(result_3, "finished", list("foo"));\
	var/result_4 = DREAMLUAU_AWAKEN(state);\
	assert_result(result_4, "finished", list("bar")))

TEST(yielding,
	var/result_1 = DREAMLUAU_LOAD(state, "return coroutine.yield(\"foo\")");\
	assert_result(result_1, "yield", list("foo"));\
	var/result_2 = DREAMLUAU_LOAD(state, "return coroutine.yield(\"bar\")");\
	assert_result(result_2, "yield", list("bar"));\
	var/result_3 = DREAMLUAU_RESUME(state, 1, list("baz"));\
	assert_result(result_3, "finished", list("baz"));\
	var/result_4 = DREAMLUAU_RESUME(state, 0, list("quux"));\
	assert_result(result_4, "finished", list("quux")))

/proc/get_wrapper()
	return "bar"

TEST(reading,
	var/obj/O = new();\
	O.name = "foo";\
	var/result_1 = DREAMLUAU_LOAD(state, "function read(thing, var) return thing\[var\] end");\
	assert_result(result_1, "finished", 0);\
	var/result_2 = DREAMLUAU_CALL_FUNCTION(state, list("read"), list(O, "name"));\
	assert_result(result_2, "finished", list("foo"));\
	DREAMLUAU_SET_VAR_GET_WRAPPER("get_wrapper");\
	var/result_3 = DREAMLUAU_CALL_FUNCTION(state, list("read"), list(O, "name"));\
	assert_result(result_3, "finished", list("bar")),
	DREAMLUAU_SET_VAR_GET_WRAPPER(null);)

/proc/set_wrapper(datum/D, var_name)
	D.vars[var_name] = "baz"

TEST(writing,
	var/obj/O = new();\
	var/result_1 = DREAMLUAU_LOAD(state, "function write(thing, var, value) thing\[var\] = value end");\
	assert_result(result_1, "finished", 0);\
	var/result_2 = DREAMLUAU_CALL_FUNCTION(state, list("write"), list(O, "name", "foo"));\
	assert_result(result_2, "finished", 0);\
	ASSERT_EQ(O.name, "foo");\
	DREAMLUAU_SET_VAR_SET_WRAPPER("set_wrapper");\
	var/result_3 = DREAMLUAU_CALL_FUNCTION(state, list("write"), list(O, "name", "bar"));\
	assert_result(result_3, "finished", 0);\
	ASSERT_EQ(O.name, "baz"),
	DREAMLUAU_SET_VAR_SET_WRAPPER(null);)

TEST(variants,
	var/result_1 = DREAMLUAU_LOAD(state, "return function() end, coroutine.create(function() end)");\
	assert_result(result_1, "finished", variants = list("function", "thread"));\
	var/result_2 = DREAMLUAU_LOAD(state, "return {\"foo\", \"bar\", \"baz\"}");\
	assert_result(result_2, "finished", list("foo", "bar", "baz")))