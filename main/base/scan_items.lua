local expect = require "cc.expect"

PUBLISH_DATA_TIME = 5
WEBSOCKET_RECONNECT_TIME = 5

function Main(input_storage, monitor)
    expect(1, input_storage, "table")
    expect(2, monitor, "table")


    local publish_data_timer_id
    local websocket_reconnect_timer_id
    local ws_handle
    http.websocketAsync("ws://127.0.0.1:3000/ws/monitor", {}, 2)
    while true do
        local eventData = {os.pullEventRaw()}
        local event = eventData[1]

        if event == "timer" and eventData[2] == publish_data_timer_id then
            SendInventory(ws_handle, input_storage)
            publish_data_timer_id = os.startTimer(PUBLISH_DATA_TIME)
        elseif event == "timer" and eventData[2] == websocket_reconnect_timer_id then
            http.websocketAsync("ws://127.0.0.1:3000/ws/monitor", {}, 2)
        elseif event == "websocket_failure" then
            print("FAIL websocket", eventData[2], eventData[3])
            if publish_data_timer_id then
                os.cancelTimer(publish_data_timer_id)
            end
            websocket_reconnect_timer_id = os.startTimer(WEBSOCKET_RECONNECT_TIME)
        elseif event == "websocket_closed" then
            print("CLOSED websocket", eventData[2], eventData[3], eventData[4])
            if publish_data_timer_id then
                os.cancelTimer(publish_data_timer_id)
            end
            websocket_reconnect_timer_id = os.startTimer(WEBSOCKET_RECONNECT_TIME)
        elseif event == "websocket_message" then
            print("MESSAGE websocket", eventData[2], eventData[3])
        elseif event == "websocket_success" then
            ws_handle = eventData[3]
            SendMonitorSize(ws_handle, monitor)
            SendInventory(ws_handle, input_storage)
            publish_data_timer_id = os.startTimer(WEBSOCKET_RECONNECT_TIME)
        elseif event == "monitor_resize" then
            print("RESIZE monitor", eventData[2])
            SendMonitorSize(ws_handle, monitor)
        elseif event == "terminate" then
            print("TERMINATE")
            os.cancelTimer(publish_data_timer_id)
            os.cancelTimer(websocket_reconnect_timer_id)
            if ws_handle then
                ws_handle.close()
            end
            return
        end
    end
end

function SendInventory(ws_handle, input_storage)
    local serialized_inventory = SerializeInventory(input_storage)
    ws_handle.send(serialized_inventory)
end

--[[
    Sends the size of the monitor to the websocket server, clears the monitor, and sets the text scale and cursor pos
    @param ws_handle: The websocket handle
    @param monitor: The monitor peripheral
]]--
function SendMonitorSize(ws_handle, monitor)
    monitor.setTextScale(0.5)
    monitor.setCursorPos(1, 1)
    monitor.setTextColor(colors.white)
    monitor.setBackgroundColor(colors.black)
    monitor.clear()
    local width, height = monitor.getSize()
    local data = "{\"monitor_resize\":{\"width\":" .. width .. ",\"height\":" .. height .. "}}"
    ws_handle.send(data)
end


function SerializeInventory(inventory)
    expect(1, inventory, "table")
    local data = "{\"inventory_report\":{"
    data = data .. "\"common_name\":\"" .. inventory.common_name .. "\","
    data = data .. "\"peripheral_name\": \"" .. inventory.peripheral_name .. "\","
    data = data .. "\"computer_id\":" .. inventory.computer_id .. ","
    if inventory.inventory_type == "storage" then
        data = data .. "\"inventory_type\":\"".. inventory.inventory_type .. "\","
    elseif inventory.inventory_type == "input" then
        data = data .. "\"inventory_type\":{\"".. inventory.inventory_type .. "\": {\"destination\":\"" .. inventory.destination .."\"}},"
    elseif inventory.inventory_type == "output" then
        data = data .. "\"inventory_type\":{\"".. inventory.inventory_type .. "\": {\"source\":\"" .. inventory.source .."\"}},"
    else 
        print("Bad inventory type", inventory.inventory_type)
    end

    data = data .. "\"inventory\":["
    for k,v in pairs(inventory.list()) do
        data = data.. "{\"slot\":" .. k ..",\"name\":\"" .. v.name .. "\", \"count\":".. v.count .. "},"
    end
    data = string.sub(data, 1, #data - 1)
    data = data .. "]}}"
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

Main(
    GetStorageInputPeripheral("MiningInput", "functionalstorage:controller_extension_0", "MainStorage"),
    peripheral.find("monitor")
)
