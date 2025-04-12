require "new_utils"

FLUX_SLOT = 15
REDSTONE_SLOT = 14
BUILDER_SLOT = 13
DIM_CHEST_SLOT = 12

IDLE_POSITION = {
    x = 1,
    y = 2,
    z = 1
}



function Main()
    LoadPositionFromLine()
    GotoPoint(IDLE_POSITION, {"y", "x", "z"})
    Orient(1)
    PickUpMachines()
    while true do
        GotoPoint(IDLE_POSITION, {"y", "x", "z"})
        PlaceMachines()
        os.sleep(18 * 60)
        PickUpMachines()
        GoToNextChunk()
    end
end


function PickUpMachines()
    turtle.select(REDSTONE_SLOT)
    turtle.digDown()
    TurnRight()
    turtle.select(DIM_CHEST_SLOT)
    turtle.dig()
    MoveDown()
    turtle.select(BUILDER_SLOT)
    turtle.dig()
    TurnLeft()
    MoveForward()
    TurnRight()
    turtle.select(FLUX_SLOT)
    turtle.dig()
end

function PlaceMachines()
    MoveForward()
    TurnRight()
    turtle.select(FLUX_SLOT)
    turtle.place()
    TurnRight()
    MoveForward()
    TurnLeft()
    turtle.select(BUILDER_SLOT)
    turtle.place()
    MoveUp()
    turtle.select(DIM_CHEST_SLOT)
    turtle.place()
    TurnLeft()
    turtle.select(REDSTONE_SLOT)
    turtle.placeDown()
    -- weird behavior where the first time we place the block, it doesnt trigger the builder, so just do it twice
    os.sleep(5)
    turtle.digDown()
    turtle.placeDown()
end


function GoToNextChunk()
    Orient(1)
    for i = 1, 16, 1  do
        MoveForward()
    end
    MoveDown()
    ResetPosition()
end