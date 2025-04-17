



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
    local data = "{"
    local data = data .. "\"common_name\":\"" .. inventory.common_name .. "\","
    local data = data .. "\"peripheral_name\": \"" .. inventory.peripheral_name .. "\","
    local data = data .. "\"computer_id\":" .. inventory.computer_id .. ","
    if inventory.inventory_type == "storage" then
        local data = data .. "\"inventory_type\":\"".. inventory.inventory_type .. "\","
    elseif inventory.inventory_type == "input" then
        local data = data .. "\"inventory_type\":{\"".. inventory.inventory_type .. "\": {\"destination\":" .. inventory.destination .."\"}},"
    elseif inventory.inventory_type == "output" then
        local data = data .. "\"inventory_type\":{\"".. inventory.inventory_type .. "\": {\"source\":" .. inventory.source .."\"}},"
    end
    local data = data .. "\"inventory_type\":\"".. inventory.inventory_type .. "\","

    local data = data .. "\"inventory\":["
    for k,v in pairs(inventory.list()) do
        data = data.. "{\"slot\":" .. k ..",\"name\":\"" .. v.name .. "\", \"count\":".. v.count .. "},"
    end
    data = string.sub(data, 1, #data - 1)
    data = data .. "]}"
    return data
end

function GetStoragePeripheral(common_name, peripheral_name)
    expect(1, common_name, "string")
    expect(2, peripheral_name, "string")

    local peripheral = peripheral.wrap(peripheral_name)
    peripheral.peripheral_name = peripheral_name
    peripheral.common_name = common_name
    peripheral.computer_id = os.getComputerID()
    peripheral.inventory_type = "storage"
    return peripheral
end

function GetStorageInputPeripheral(common_name, peripheral_name, destination)
    expect(1, common_name, "string")
    expect(2, peripheral_name, "string")
    expect(3, destination, "string")

    local peripheral = GetStoragePeripheral(common_name, peripheral_name)
    peripheral.inventory_type = "input"
    peripheral.destination = destination
    return peripheral
end


function GetStorageOutputPeripheral(common_name, peripheral_name, source)
    expect(1, common_name, "string")
    expect(2, peripheral_name, "string")
    expect(3, source, "string")

    local peripheral = GetStoragePeripheral(common_name, peripheral_name)
    peripheral.inventory_type = "output"
    peripheral.source = source
    return peripheral
end

Main(GetStorageInputPeripheral("MiningInput", "functionalstorage:controller_extension_0", "MainStorage"))
