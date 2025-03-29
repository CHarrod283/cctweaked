
Position = {
    direction = 'n',
    x = 1,
    y = 253,
    z = 1,
}


NextMiningBlock = {
    x = 1,
    y = 252,
    z = 1,
}


function Main()
    while true do
        Mine()
    end
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
    local mining_plane = false
    while OkToMine() do
        -- if were above our block
        if Position.y - 1 == NextMiningBlock.y then
            if Position.y == 1 then
                NextMiningBlock = nil
                return
            end
            turtle.digDown()
            MoveDown()
            NextMiningBlock.z = NextMiningBlock.z + 1
            mining_plane = true
        elseif mining_plane then
            turtle.dig()
            MoveForward()
            if Position.x >= 16 and Position.z <= 1 then
                mining_plane = false
                NextMiningBlock.x = 1
                NextMiningBlock.z = 1
                NextMiningBlock.y = Position.y - 1
            elseif Position.direction == "n" and Position.z >= 16 then
                TurnRight()
                NextMiningBlock.x = NextMiningBlock.x + 1
            elseif Position.direction == "s" and Position.z <= 1 then
                TurnLeft()
                NextMiningBlock.x = NextMiningBlock.x + 1
            elseif Position.direction == "e" then
                TurnRight()
                NextMiningBlock.z = NextMiningBlock.z - 1
            elseif Position.direction == "w" then
                TurnLeft()
                NextMiningBlock.z = NextMiningBlock.z + 1
            end
        else
            -- go to starting point of next plane
            if Position.direction == "s" then
                TurnRight()
                MoveForward()
            elseif Position.x > 1 then
                MoveForward()
            else
                TurnRight()
            end
        end
    end
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

function MoveBack()
    if turtle.back() then
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

function HaveEnoughFuel()
    return turtle.getFuelLevel() > 1000
end


function HaveInventorySpace()
    for i = 1, 16, 1 do
        turtle.select(i)
        if turtle.getItemSpace() == 64 then
            return true
        end
    end
end


Mine()