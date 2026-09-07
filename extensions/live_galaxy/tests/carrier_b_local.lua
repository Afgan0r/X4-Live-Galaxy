local expected = ...
assert(type(expected) == "string" and expected ~= "", "actual result path required")
local result = assert(loadfile(expected))()
assert(result.actual_native == true, "actual owned native module required")
assert(result.durable_revision == 1, "durable revision 1 required")
return true
