local expect = require "cc.expect"
local pretty = require "cc.pretty"


DISPLAY_WIDTH = 5

function Main(input_storage, monitor, iteration_speed, redstone)
    expect(1, input_storage, "table")
    expect(2, monitor, "table")
    expect(3, iteration_speed, "number")
    expect(4, redstone, "table")

    local monitor_x, monitor_y = monitor.getSize()
    local box_x = monitor_x / DISPLAY_WIDTH

    while true do
        local input_items = input_storage.list()
        local working_box_x = 0
        local working_box_y = 0
        for k,v in pairs(input_items) do
            monitor.setCursorPos(working_box_x * box_x + 1, working_box_y + 1)
            monitor.write("+")
            for i = 2, box_x - 1, 1 do
                monitor.write("-")
            end
            monitor.write("+")

            monitor.setCursorPos(working_box_x * box_x + 1, working_box_y  + 2)
            monitor.write("|")
            local formatted_data = string.format("%s = %d", v.name, v.count)
            for i = #formatted_data, box_x - 1, 1 do
                monitor.write(" ")
            end
            monitor.write("|")


            monitor.setCursorPos(working_box_x * box_x + 1, working_box_y + 1)
            monitor.write("+")
            for i = 2, box_x - 1, 1 do
                monitor.write("-")
            end
            monitor.write("+")


            if working_box_x == DISPLAY_WIDTH - 1 then
                working_box_y = working_box_y + 1
            end
        end
    end
end


