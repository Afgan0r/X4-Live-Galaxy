-- Order owned observation arrays inside the source callback. The closure
-- preserves the collector interface without scheduling artificial pauses.
local order = {}
function order.new(rows, less)
    return function()
        table.sort(rows, less)
        return true
    end
end
return order
