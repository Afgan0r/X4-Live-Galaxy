-- Incremental merge ordering for owned observation arrays. Each comparison or
-- copy is charged; no callback performs an entire O(n log n) sort.
local order = {}
function order.new(rows, less)
    local run = coroutine.create(function()
        local scratch, width, count = {}, 1, #rows
        while width < count do
            for first = 1, count, width * 2 do
                local middle, last = math.min(first + width, count + 1), math.min(first + width * 2 - 1, count)
                local left, right = first, middle
                for target = first, last do
                    if left < middle and (right > last or not less(rows[right], rows[left])) then
                        scratch[target], left = rows[left], left + 1
                    else scratch[target], right = rows[right], right + 1 end
                    coroutine.yield()
                end
                for target = first, last do rows[target] = scratch[target]; coroutine.yield() end
            end
            width = width * 2
        end
    end)
    return function()
        for _ = 1, 32 do
            local ok, err = coroutine.resume(run)
            if not ok then return nil, err end
            if coroutine.status(run) == "dead" then return true end
        end
        return false
    end
end
return order
