require "utils"

local expect = require "cc.expect"




function Main(player_detector, proximity, timeout)
    expect(1, player_detector, "table")
    expect(2, proximity, "number")
    expect(3, timeout, "number")
    while true do
        os.sleep(timeout)
        if not player_detector.isPlayerInRange(20, "WoodArrow") then
            
            redstone.setOutput("top", false)
        else
            redstone.setOutput("top", true)
        end
    end
end

Main(peripheral.find("playerDetector"), 20, 1)