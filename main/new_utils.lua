DATA_FILE = "data.txt"

REFUEL_SLOT = 16

Position = {
    direction = 1, -- 1 is just the direction the turtle is facing when it is placed down
    x = 1,
    y = 1,
    z = 1
}


function ResetPosition()
    Position.direction = 1
    Position.x = 1
    Position.y = 1
    Position.z = 1
    SavePositionAsLine()
end

--[[
Writes the position to a string, this can later be loaded by LoadPositionFromLine()
]]--
function SavePositionAsLine()
    local data = string.format("%d %d %d %d", Position.direction, Position.x, Position.y, Position.z)
    local write_file, err = io.open(DATA_FILE, "w+")
    if write_file then
        write_file:write(data)
        write_file:flush()
        write_file:close()
    else
        print("err:", err)
    end
end

--[[
Loads the position from a line, this line can be generated by PositionAsLine()
]]
function LoadPositionFromLine()
    local read_file = io.open(DATA_FILE, "r")
    if not read_file then
        return -- no file to read, so do nothing
    end
    local data = read_file:read("a")
    read_file:close()
    local i = 1
    for s in string.gmatch(data, "%S+") do
        if i == 1 then
            Position.direction = tonumber(s)
        elseif i == 2 then
            Position.x = tonumber(s)
        elseif i == 3 then
            Position.y = tonumber(s)
        elseif i == 4 then
            Position.z = tonumber(s)
        else
            print("Weird file bro")
            os.exit(-1)
        end
        i = i + 1
    end
end


function Refuel()
    turtle.select(REFUEL_SLOT)
    turtle.placeUp()
    turtle.suckUp(10)
    turtle.refuel()
    turtle.digUp()
    turtle.select(1)
end


function TurnLeft()
    turtle.turnLeft()
    Position.direction = Position.direction - 1
    if Position.direction == 0 then
        Position.direction = 4
    end
    SavePositionAsLine()
end

function TurnRight()
    turtle.turnRight()
    Position.direction = Position.direction + 1
    if Position.direction == 5 then
        Position.direction = 1
    end
    SavePositionAsLine()
end

function CheckToRefuel()
    if turtle.getFuelLevel() == 0 then
        Refuel()
    end
end

function MoveForward()
    CheckToRefuel()
    if not turtle.forward() then
        return
    end
    if Position.direction == 1 then
        Position.x = Position.x + 1
    elseif Position.direction ==2 then
        Position.z = Position.z + 1
    elseif Position.direction == 3 then
        Position.x = Position.x - 1
    elseif Position.direction == 4 then
        Position.z = Position.z - 1
    else
        print("weird position bro")
        os.exit(1)
    end
    SavePositionAsLine()
end

function MoveBack()
    CheckToRefuel()
    if not turtle.back() then
        return
    end
    if Position.direction == 1 then
        Position.x = Position.x - 1
    elseif Position.direction ==2 then
        Position.z = Position.z - 1
    elseif Position.direction == 3 then
        Position.x = Position.x + 1
    elseif Position.direction == 4 then
        Position.z = Position.z + 1
    else
        print("weird position bro")
        os.exit(1)
    end
    SavePositionAsLine()
end

function MoveUp()
    CheckToRefuel()
    if not turtle.up() then
        return
    end
    Position.y = Position.y + 1
    SavePositionAsLine()
end

function MoveDown()
    CheckToRefuel()
    if not turtle.down() then
        return
    end
    Position.y = Position.y - 1
    SavePositionAsLine()
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
        Orient(1)
    elseif Position.x > position.x then
        Orient(3)
    end
    while Position.x ~= position.x do
        MoveForward()
    end
end

function GotoZ(position)
    if Position.z > position.z then
        Orient(2)
    elseif Position.z < position.z then
        Orient(4)
    end
    while Position.z ~= position.z do
        MoveForward()
    end
end


function Orient(direction)
    if Position.direction == direction then
        return
    end
    if direction == 1 then
        if Position.direction == 3 then
            TurnRight()
            TurnRight()
        elseif Position.direction == 2 then
            TurnLeft()
        elseif Position.direction == 4 then
            TurnRight()
        end
    elseif direction == 2 then
        if Position.direction == 4 then
            TurnRight()
            TurnRight()
        elseif Position.direction == 3 then
            TurnLeft()
        elseif Position.direction == 1 then
            TurnRight()
        end
    elseif direction == 3 then
        if Position.direction == 1 then
            TurnRight()
            TurnRight()
        elseif Position.direction == 2 then
            TurnRight()
        elseif Position.direction == 4 then
            TurnLeft()
        end
    elseif direction == 4 then
        if Position.direction == 2 then
            TurnRight()
            TurnRight()
        elseif Position.direction == 3 then
            TurnRight()
        elseif Position.direction == 1 then
            TurnLeft()
        end
    end
end