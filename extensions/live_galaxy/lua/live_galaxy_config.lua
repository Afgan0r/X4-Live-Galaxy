-- The source tree defaults to the established clock profile. Packaging replaces
-- this composition module with the exact validated experimental profile.
local config = {}
function config.options() return { observation = { profile = "carrier_b_realtime_sample" } } end
return config
