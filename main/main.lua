
require "utils"

ORIGIN = {
    x = 1,
    y = 253,
    z = 1
}

MIN_Y_POSITION = -63

MAX_Y_POSITION = 252

MINE_SIZE = 16
MINE_DIRECTION = 'e'

Position = {
    direction = 'e',
    x = ORIGIN.x,
    y = ORIGIN.y,
    z = ORIGIN.z
}

REFUEL_SLOT = 16
DUMP_INVENTORY_SLOT = 15
LAST_INVENTORY_SLOT = 14

MiningInfo = {
    x_dir = 1,
    z_dir = 1,
    y_dir = -1,
}



function Main()
    turtle.digDown()
    while true do
        Mine()
        if not HaveInventorySpace() then
            print("emptying inventory")
            EmptyInventory()
        end
        if not HaveEnoughFuel() then
            print("refueling")
            Refuel()
        end
    end
end
--[[
We are ok to mine if we are within 1 block of our mineable block in any direction and have fuel and inventory space
]]--
function OkToMine()
    return HaveEnoughFuel() and HaveInventorySpace()
end


--[[
Resumable mining function
]]--
function Mine()
    while OkToMine() do
        if
            (MiningInfo.y_dir == -1 and Position.y <= MIN_Y_POSITION or MiningInfo.y_dir == 1 and Position.y >= MAX_Y_POSITION) and
            (Position.x >= MINE_SIZE and MiningInfo.x_dir == 1 or Position.x <= 1 and MiningInfo.x_dir == -1) and
            (Position.z >= MINE_SIZE and MiningInfo.z_dir == 1 or Position.z <= 1 and MiningInfo.z_dir == -1)
        then
            Orient(MINE_DIRECTION)
            while
                (MINE_DIRECTION == "e" and Position.x < MINE_SIZE) or
                (MINE_DIRECTION == "w" and Position.x > 1) or
                (MINE_DIRECTION == "s" and Position.z < MINE_SIZE) or
                (MINE_DIRECTION == "n" and Position.z > 1)
            do
                MoveForward()
            end
            turtle.dig()
            MoveForward()
            turtle.digUp()
            turtle.digDown()

            if MiningInfo.y_dir == 1 then
                MiningInfo.y_dir = -1
            else
                MiningInfo.y_dir = 1
            end
            if Position.z ==  1 or Position.z == MINE_SIZE + 1 then
                MiningInfo.z_dir = 1
                Position.z = 1
            else
                MiningInfo.z_dir = -1
                Position.z = MINE_SIZE
            end
            if Position.x == 1 or Position.x == MINE_SIZE + 1 then
                MiningInfo.x_dir = 1
                Position.x = 1
                Orient("e")
            else 
                MiningInfo.x_dir = -1
                Position.x = MINE_SIZE
                Orient("w")
            end
        elseif
            (Position.x >= MINE_SIZE and MiningInfo.x_dir == 1 or Position.x <= 1 and MiningInfo.x_dir == -1) and
            (Position.z >= MINE_SIZE and MiningInfo.z_dir == 1 or Position.z <= 1 and MiningInfo.z_dir == -1)
        then
            if MiningInfo.y_dir == 1 then
                MoveUp()
                turtle.digUp()
                MoveUp()
                turtle.digUp()
                MoveUp()
                turtle.digUp()
                TurnRight()
                TurnRight()
            else 
                MoveDown()
                turtle.digDown()
                MoveDown()
                turtle.digDown()
                MoveDown()
                turtle.digDown()
                TurnRight()
                TurnRight()
            end
            if MiningInfo.x_dir == 1 then
                MiningInfo.x_dir = -1
            else
                MiningInfo.x_dir = 1
            end
            if MiningInfo.z_dir == 1 then
                MiningInfo.z_dir = -1
            else
                MiningInfo.z_dir = 1
            end
        elseif Position.x >= MINE_SIZE and MiningInfo.x_dir == 1 or Position.x <= 1 and MiningInfo.x_dir == -1 then
            if MiningInfo.z_dir == 1 then
                if MiningInfo.x_dir == 1 then
                    TurnRight()
                    turtle.dig()
                    MoveForward()
                    turtle.digUp()
                    turtle.digDown()
                    TurnRight()
                else
                    TurnLeft()
                    turtle.dig()
                    MoveForward()
                    turtle.digUp()
                    turtle.digDown()
                    TurnLeft()
                end
            else
                if MiningInfo.x_dir == 1 then
                    TurnLeft()
                    turtle.dig()
                    MoveForward()
                    turtle.digUp()
                    turtle.digDown()
                    TurnLeft()
                else
                    TurnRight()
                    turtle.dig()
                    MoveForward()
                    turtle.digUp()
                    turtle.digDown()
                    TurnRight()
                end
            end


            if MiningInfo.x_dir == 1 then
                MiningInfo.x_dir = -1
            else
                MiningInfo.x_dir = 1
            end
        else
            turtle.dig()
            MoveForward()
            turtle.digUp()
            turtle.digDown()
        end
    end
end

function DebugGlobals()
    --local debug_output = "Position: " + Position.x + ", " + Position.y + ", " + Position.z + "\n"
    --debug_output = debug_output + "NextMiningBlock " + MiningOrder.NextMiningBlock.x +", " + MiningOrder.NextMiningBlock.y +", " +  MiningOrder.NextMiningBlock.z
    return "uh oh"
end

function Refuel()
    turtle.select(REFUEL_SLOT)
    turtle.placeUp()
    turtle.suckUp(10)
    turtle.refuel()
    turtle.digUp()
    turtle.select(1)
end



function EmptyInventory()
    turtle.digUp() -- ensure we can place our block
    turtle.select(DUMP_INVENTORY_SLOT)
    turtle.placeUp()
    for i = 1, LAST_INVENTORY_SLOT, 1 do
        turtle.select(i)
        turtle.dropUp()
    end
    turtle.select(DUMP_INVENTORY_SLOT)
    turtle.digUp()
    turtle.select(1)
end




function HaveEnoughFuel()
    return turtle.getFuelLevel() > 1000
end


function HaveInventorySpace()
    turtle.select(LAST_INVENTORY_SLOT)
    if turtle.getItemSpace() == 64 then
        turtle.select(1)
        return true
    end
    turtle.select(1)
    return false
end

Main()