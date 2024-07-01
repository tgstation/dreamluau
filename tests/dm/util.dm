#define ASSERT_EQ_MSG(l, r, msg) if(##l != ##r) \
{\
	throw EXCEPTION("Assertion failed: [##msg]");\
}

#define ASSERT_EQ(l, r) ASSERT_EQ_MSG(l, r, "[#l] == [r]")

#define ASSERT_FINDTEXT(value, needle, msg) if(!findtext(needle, value)) \
{\
	throw EXCEPTION("Assertion failed: [##msg]");\
}

/proc/deep_compare_list(list/list_1, list/list_2, index_name = "")
	if(list_1 == list_2)
		return TRUE

	if(!islist(list_1) || !islist(list_2))
		return FALSE

	if(list_1.len != list_2.len)
		return FALSE

	for(var/i in 1 to list_1.len)
		var/key_1 = list_1[i]
		var/key_2 = list_2[i]
		if (islist(key_1) && islist(key_2))
			deep_compare_list(key_1, key_2, "[index_name]\[[i]\]")
		else
			ASSERT_EQ_MSG(key_1, key_2, "[index_name]\[[i]\] == [key_2]")
		if(istext(key_1) || islist(key_1) || ispath(key_1) || istype(key_1, /datum) || key_1 == world)
			var/value_1 = list_1[key_1]
			var/value_2 = list_2[key_1]
			if (islist(value_1) && islist(value_2))
				deep_compare_list(value_1, value_2, "[index_name]\[[key_1]\]")
			else
				ASSERT_EQ_MSG(value_1, value_2, "[index_name]\[[key_1]\] == [value_2]")
	return TRUE

/proc/assert_result(result, status, values, variants, errmsg)
	if(istext(result))
		throw EXCEPTION(result)
	ASSERT(islist(result))
	if(status)
		ASSERT_EQ_MSG(result["status"], status, "expexted status of \"[status]\", got \"[result["status"]]\"")
	if(status == "error")
		ASSERT_FINDTEXT(result["message"], errmsg, "expected error message containing \"[errmsg]\", got \"[result["message"]]\"")
	if(isnum(values))
		ASSERT_EQ(length(result["return_values"]), values)
	else if(islist(values))
		deep_compare_list(result["return_values"], values, "return values")
	if(islist(variants))
		deep_compare_list(result["variants"], variants, "variants")