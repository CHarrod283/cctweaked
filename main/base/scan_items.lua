



local expect = require "cc.expect"

function Main(input_storage_drawer)
    expect(1, input_storage_drawer, "table")

    while true do
        os.sleep(1)
        local serialized_inventory = SerializeInventory(input_storage_drawer)
        local headers = {
            ["content-type"] = "application/json"
        }
        http.request("http://127.0.0.1:3000", serialized_inventory, headers)
        while true do
            local eventData = {os.pullEvent()}
            local event = eventData[1]
            if event == "http_success" then
                break
            elseif event == "http_failure" then
                print("FAIL http")
                break
            end
        end
    end
end


function SerializeInventory(inventory)
    expect(1, inventory, "table")
    local data = "{\"peripheral_name\": \"" .. inventory.peripheral_name .. "\",\"computer_id\": " .. inventory.computer_id .. ",\"inventory_type\":".. inventory.inventory_type .. "\", \"inventory\":["
    
    for k,v in pairs(inventory.list()) do
        data = data.. "{\"slot\": " .. k ..", \"name\":\"" .. v.name .. "\", \"count\": ".. v.count .. "},"
    end
    data = string.sub(data, 1, #data - 1)
    data = data .. "]}"
    return data
end

function GetStoragePeripheral(name, type)
    expect(1, name, "string")
    expect(2, type, "string")

    local peripheral = peripheral.wrap(name)
    peripheral.peripheral_name = name
    peripheral.computer_id = os.getComputerID()
    peripheral.inventory_type = type
    return peripheral
end

Main(GetStoragePeripheral("functionalstorage:controller_extension_0", "input"))
