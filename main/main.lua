
Position = {
    direction = 'e',
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

MiningOrder = {
    START = {
        x = 1,
        y = 253,
        z = 1,
    },
    METHOD = {
        "x",
        "z",
        "y"
    },
    DIMENSIONS = {
        x = 5,
        y = 5,
        z = 5
    },
    RelativeNextMiningBlock = {
        x = 1,
        y = 1,
        z = 1,
    }
}

function Main()
    while MiningOrder.RelativeNextMiningBlock ~= nil do
        MineV2()
        if not HaveInventorySpace() then
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
    local mining_plane = false
    while OkToMine() do
        -- if were above our block
        if Position.y - 1 == NextMiningBlock.y and Position.x == NextMiningBlock.x and Position.z == NextMiningBlock.z then
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
            elseif Position.direction == "e" and Position.z >= 16 then
                TurnRight()
                NextMiningBlock.z = NextMiningBlock.z - 1
            elseif Position.direction == "e" and Position.z <= 1 then
                TurnLeft()
                NextMiningBlock.z = NextMiningBlock.z + 1
            elseif Position.direction == "n" then
                NextMiningBlock.z = NextMiningBlock.z + 1
            elseif Position.direction == "s" then
                NextMiningBlock.z = NextMiningBlock.z - 1
            end
        else
            -- go to starting point of next plane
            if Position.direction == "s" then
                TurnRight()
                MoveForward()
            elseif Position.x > 1 then
                MoveForward()
            end
            if Position.x <= 1 then
                TurnRight()
            end
        end
    end
end

--[[
Resumable mining function
]]--
function MineV2()
    assert(Position.x == MiningOrder.START.x and Position.y == MiningOrder.START.y and Position.z == MiningOrder.START.z, "Not at starting mine position ", DebugGlobals())
    if (MiningOrder.DIMENSIONS.x == 0 or MiningOrder.DIMENSIONS.y == 0 or MiningOrder.DIMENSIONS.z == 0) then
        return -- nothing to do
    end

    -- if we arent next to our next mining block, go to above the next mining block
    local next_mining_block = GetPoint(MiningOrder.START, MiningOrder.RelativeNextMiningBlock)
    local point_above_next_mining_block = GetPoint(next_mining_block, {x = 0, y = 1, z = 0})
    while OkToMine() do
        MoveTowardsPoint(point_above_next_mining_block, {"x", "z", "y"})
    end
    -- mine
    local i = 0
    while OkToMine() and i < 10 do
        i = i + 1
        print("getting mining block")
        next_mining_block = GetPoint(MiningOrder.START, MiningOrder.RelativeNextMiningBlock)
        print("NextMiningBlock", next_mining_block.x, next_mining_block.y, next_mining_block.z)
        MineBlock()
        print("MinedBlock")
        MoveTowardsPoint(next_mining_block, MiningOrder.METHOD)
        UpdateRelativeNextMiningBlock()
    end
end

function UpdateRelativeNextMiningBlock()
    local encoded_block = (MiningOrder.RelativeNextMiningBlock["y"] - 1) * MiningOrder.DIMENSIONS["z"] * MiningOrder.DIMENSIONS["x"]
    if MiningOrder.RelativeNextMiningBlock["y"] % 2 == 0 then
        encoded_block = encoded_block + (MiningOrder.DIMENSIONS["z"] - MiningOrder.RelativeNextMiningBlock["z"]) * MiningOrder.DIMENSIONS["x"]
    else
        encoded_block = encoded_block + (MiningOrder.RelativeNextMiningBlock["z"] - 1) * MiningOrder.DIMENSIONS["x"]
    end
    if MiningOrder.RelativeNextMiningBlock["z"] % 2 == 0 then
        encoded_block = encoded_block + (MiningOrder.DIMENSIONS["x"] - MiningOrder.RelativeNextMiningBlock["x"])
    else
        encoded_block = encoded_block + (MiningOrder.RelativeNextMiningBlock["x"] - 1)
    end

    local can_move_x = encoded_block % MiningOrder.DIMENSIONS["x"] ~= 0
    local can_move_z = encoded_block % (MiningOrder.DIMENSIONS["x"] * MiningOrder.DIMENSIONS["z"]) ~= 0
    local can_move_y = encoded_block % (MiningOrder.DIMENSIONS["x"] * MiningOrder.DIMENSIONS["z"] * MiningOrder.DIMENSIONS["y"]) ~= 0
    if can_move_x then
        if MiningOrder.RelativeNextMiningBlock.z % 2 == 0 then
            if MiningOrder.RelativeNextMiningBlock.y % 2 == 0 then
                MiningOrder.RelativeNextMiningBlock.x = MiningOrder.RelativeNextMiningBlock.x + 1
            else
                MiningOrder.RelativeNextMiningBlock.x = MiningOrder.RelativeNextMiningBlock.x - 1
            end
        else
            if MiningOrder.RelativeNextMiningBlock.y % 2 == 0 then
                MiningOrder.RelativeNextMiningBlock.x = MiningOrder.RelativeNextMiningBlock.x - 1
            else
                MiningOrder.RelativeNextMiningBlock.x = MiningOrder.RelativeNextMiningBlock.x + 1
            end
        end
        return
    end
    if can_move_z then
        if MiningOrder.RelativeNextMiningBlock.y % 2 == 0 then
            MiningOrder.RelativeNextMiningBlock.x = MiningOrder.RelativeNextMiningBlock.x - 1
        else
            MiningOrder.RelativeNextMiningBlock.x = MiningOrder.RelativeNextMiningBlock.x + 1
        end
    end
    if can_move_y then
        MiningOrder.RelativeNextMiningBlock.y = MiningOrder.RelativeNextMiningBlock - 1
    end
    MiningOrder.RelativeNextMiningBlock = nil
end


function MineBlock()
    local next_mining_block = GetPoint(MiningOrder.START, MiningOrder.RelativeNextMiningBlock)
    FaceBlock(next_mining_block, {"x", "z", "y"})
    local dist_x = math.abs(Position.x - next_mining_block.x)
    local dist_y = math.abs(Position.y - next_mining_block.y)
    local dist_z = math.abs(Position.z - next_mining_block.z)
    assert(dist_x + dist_y + dist_z == 1, "Not close enough to block", dist_x, dist_y, dist_z, DebugGlobals())
    if dist_y == 1 then
        turtle.digDown()
    else
        turtle.dig()
    end
end


function DebugGlobals()
    --local debug_output = "Position: " + Position.x + ", " + Position.y + ", " + Position.z + "\n"
    --debug_output = debug_output + "NextMiningBlock " + MiningOrder.NextMiningBlock.x +", " + MiningOrder.NextMiningBlock.y +", " +  MiningOrder.NextMiningBlock.z
    return "uh oh"
end

function Refuel()
    assert(false, "Refuel Unimplemented")
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
    assert( false, "EmptyInventory not implemented")
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