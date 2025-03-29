
Position = {
    direction = 'e',
    x = 1,
    y = 253,
    z = 1,
}

LastMiningPosition = nil

NextMiningBlock = {
    x = 1,
    y = 252,
    z = 1,
    mining_plane = false,
}

MINE_START = {
    x = 1,
    y = 253,
    z = 1,
}

ORIGIN = {
    x = 1,
    y = 253,
    z = 1
}

FUEL_DEPO = {
    x = 0,
    y = 253,
    z = 1
}
INVENTORY_DROPOFF = {
    x = 0,
    y = 253,
    z = 2
}



function Main()
    while NextMiningBlock ~= nil do
        GotoPoint(MINE_START, {"x", "y", "z"})
        Mine()
        if not HaveInventorySpace() then
            print("emptying inventory")
            EmptyInventory()
        end
        if not HaveEnoughFuel() then
            Refuel()
        end
    end
    GotoPoint(ORIGIN, {"y", "x", "z"})
    Orient("s")
end
--[[
We are ok to mine if we are within 1 block of our mineable block in any direction and have fuel and inventory space
]]--
function OkToMine()
    if NextMiningBlock == nil then
        return false
    end
    return HaveEnoughFuel() and HaveInventorySpace()
end


--[[
Resumable mining function
]]--
function Mine()
    if LastMiningPosition ~= nil then
        GotoPoint(LastMiningPosition, {"x", "z", "y"})
        Orient(LastMiningPosition.direction)
    end
    
    while OkToMine() do
        print("Position: ", Position.x , Position.y, Position.z)
        print("NextMiningBlock", NextMiningBlock.x, NextMiningBlock.y, NextMiningBlock.z)
        -- if were above our block
        if Position.y - 1 == NextMiningBlock.y and Position.x == NextMiningBlock.x and Position.z == NextMiningBlock.z then
            if Position.y == 1 then
                NextMiningBlock = nil
                return
            end
            turtle.digDown()
            MoveDown()
            NextMiningBlock.x = NextMiningBlock.x + 1
            Orient("e")
            NextMiningBlock.mining_plane = true
        elseif NextMiningBlock.mining_plane then
            turtle.dig()
            MoveForward()
            if Position.z >= 16 and Position.x <= 1 then
                NextMiningBlock.mining_plane = false
                NextMiningBlock.x = 1
                NextMiningBlock.z = 1
                NextMiningBlock.y = Position.y - 1
            elseif Position.direction == "e" and Position.x >= 16 then
                TurnRight()
                NextMiningBlock.z = NextMiningBlock.z + 1
            elseif Position.direction == "w" and Position.x <= 1 then
                TurnLeft()
                NextMiningBlock.z = NextMiningBlock.z + 1
            elseif Position.direction == "s" and Position.x >= 16 then
                TurnRight()
                NextMiningBlock.x = NextMiningBlock.x - 1
            elseif Position.direction == "s" and Position.x <= 1 then
                TurnLeft()
                NextMiningBlock.x = NextMiningBlock.x + 1
            elseif Position.direction == "e" then
                NextMiningBlock.x = NextMiningBlock.x + 1
            elseif Position.direction == "w" then
                NextMiningBlock.x = NextMiningBlock.x - 1
            end
        else
            -- go to starting point of next plane
            if Position.direction == "w" then
                TurnRight()
                MoveForward()
            elseif Position.z > 1 then
                MoveForward()
            end
            if Position.z <= 1 then
                TurnRight()
            end
        end
    end
    SaveLastMiningPosition()
end


function SaveLastMiningPosition()
    LastMiningPosition = {
        x = Position.x,
        y = Position.y,
        z = Position.z,
        direction = Position.direction
    }
end

function DebugGlobals()
    --local debug_output = "Position: " + Position.x + ", " + Position.y + ", " + Position.z + "\n"
    --debug_output = debug_output + "NextMiningBlock " + MiningOrder.NextMiningBlock.x +", " + MiningOrder.NextMiningBlock.y +", " +  MiningOrder.NextMiningBlock.z
    return "uh oh"
end

function Refuel()
    GotoPoint(GetPoint(FUEL_DEPO, {x = 1, y = 0, z = 0}), {"y", "z", "x"})
    Orient("w")
    turtle.select(1)
    turtle.suck(10)
    turtle.refuel()
end

--[[
Goes to a point
Position is a xyz position (i.e {1, 2,3})
Method is "how" to get there, i.e if you want to align x first then y then z you input {"x", "y", "z"}
]]--
function GotoPoint(position, method)
    for i = 1, #method, 1 do
        if method[i] == "x" then
            GotoX(position)
        elseif method[i] == "y" then
            GotoY(position)
        elseif method[i] == "z" then
            GotoZ(position)
        end
    end
end

function GotoY(position)
    while Position.y > position.y do
        MoveDown()
    end
    while Position.y < position.y do
        MoveUp()
    end
end

function GotoX(position)
    if Position.x < position.x then
        Orient("e")
    elseif Position.x > position.x then
        Orient("w")
    end
    while Position.x ~= position.x do
        MoveForward()
    end
end

function GotoZ(position)
    if Position.z > position.z then
        Orient("n")
    elseif Position.z < position.z then
        Orient("s")
    end
    while Position.z ~= position.z do
        MoveForward()
    end
end

--[[
Moves 1 block towards a point prioritized by method
]]--
function MoveTowardsPoint(position, method)
    for i = 1, #method, 1 do
        if method[i] == "x" and Position.x ~= position.x then
            MoveTowardsX(position)
            break
        elseif method[i] == "y" and Position.y ~= position.y then
            MoveTowardsY(position)
            break
        elseif method[i] == "z" and Position.z ~= position.z then
            MoveTowardsZ(position)
            break
        end
    end
end

function MoveTowardsY(position)
    if Position.y > position.y then
        MoveDown()
    end
    if Position.y < position.y then
        MoveUp()
    end
end

function MoveTowardsX(position)
    if Position.x < position.x then
        Orient("e")
    elseif Position.x > position.x then
        Orient("w")
    end
    MoveForward()
end

function MoveTowardsZ(position)
    if Position.z > position.z then
        Orient("n")
    elseif Position.z < position.z then
        Orient("s")
    end
    MoveForward()
end

function FaceBlock(position, method)
    for i = 1, #method, 1 do
        if method[i] == "x" and Position.x < position.x then
            Orient("e")
            break
        elseif method[i] == "x" and Position.x > position.x then
            Orient("w")
            break
        elseif method[i] == "z" and Position.z < position.z then
            Orient("s")
            break
        elseif method[i] == "z" and Position.z > position.z then
            Orient("n")
            break
        end
    end
end


function GetPoint(position, offset)
    return {
        x = position.x + offset.x,
        y = position.y + offset.y,
        z = position.z + offset.z
    }
end



function Orient(direction)
    if Position.direction == direction then
        return
    end
    if direction == "n" then
        if Position.direction == "s" then
            TurnRight()
            TurnRight()
        elseif Position.direction == "e" then
            TurnLeft()
        elseif Position.direction == "w" then
            TurnRight()
        end
    elseif direction == "e" then
        if Position.direction == "w" then
            TurnRight()
            TurnRight()
        elseif Position.direction == "s" then
            TurnLeft()
        elseif Position.direction == "n" then
            TurnRight()
        end
    elseif direction == "s" then
        if Position.direction == "n" then
            TurnRight()
            TurnRight()
        elseif Position.direction == "e" then
            TurnRight()
        elseif Position.direction == "w" then
            TurnLeft()
        end
    elseif direction == "w" then
        if Position.direction == "e" then
            TurnRight()
            TurnRight()
        elseif Position.direction == "s" then
            TurnRight()
        elseif Position.direction == "n" then
            TurnLeft()
        end
    end
end

function EmptyInventory()
    GotoPoint(GetPoint(INVENTORY_DROPOFF, {x = 1, y = 0, z = 0}), {"y", "z", "x"})
    Orient("w")
    for i = 1, 16, 1 do
        turtle.select(i)
        turtle.drop()
    end
    turtle.select(1)
end


function TurnRight()
    turtle.turnRight()
    if Position.direction == "n" then
        Position.direction = "e"
    elseif Position.direction == "e" then
        Position.direction = "s"
    elseif Position.direction == "s" then
        Position.direction = "w"
    elseif Position.direction == "w" then
        Position.direction = "n"
    end
end


function TurnLeft()
    turtle.turnLeft()
    if Position.direction == "n" then
        Position.direction = "w"
    elseif Position.direction == "w" then
        Position.direction = "s"
    elseif Position.direction == "s" then
        Position.direction = "e"
    elseif Position.direction == "e" then
        Position.direction = "n"
    end
end

function MoveDown()
    if turtle.down() then
        Position.y = Position.y - 1
    end
end

function MoveUp()
    if turtle.up() then
        Position.y = Position.y + 1
    end
end

function MoveForward()
    if turtle.forward() then
        if Position.direction == "n" then
            Position.z = Position.z - 1
        elseif Position.direction == "e" then
            Position.x = Position.x + 1
        elseif Position.direction == "s" then
            Position.z = Position.z + 1
        elseif Position.direction == "w" then
            Position.x = Position.x - 1
        end
    end
end

function MoveBack()
    if turtle.back() then
        if Position.direction == "n" then
            Position.z = Position.z + 1
        elseif Position.direction == "e" then
            Position.x = Position.x - 1
        elseif Position.direction == "s" then
            Position.z = Position.z - 1
        elseif Position.direction == "w" then
            Position.x = Position.x + 1
        end
    end
end

function HaveEnoughFuel()
    return turtle.getFuelLevel() > 1000
end


function HaveInventorySpace()
    turtle.select(16)
    if turtle.getItemSpace() == 64 then
        turtle.select(1)
        return true
    end
    turtle.select(1)
    return false
end

Main()